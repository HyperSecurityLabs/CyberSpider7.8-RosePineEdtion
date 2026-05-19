use anyhow::Result;
use redis::{Client, AsyncCommands};
use std::time::Duration;
use tokio::time::sleep;
use crate::database::Queue;

pub struct RedisQueue {
    client: Client,
    queue_name: String,
    max_retries: usize,
    retry_delay: Duration,
}

impl RedisQueue {
    pub async fn new(redis_url: &str, queue_name: &str) -> Result<Self> {
        let client = Client::open(redis_url)?;
        
        Ok(Self {
            client,
            queue_name: queue_name.to_string(),
            max_retries: 3,
            retry_delay: Duration::from_millis(1000),
        })
    }

    pub async fn new_with_config(
        redis_url: &str, 
        queue_name: &str,
        max_retries: usize,
        retry_delay_ms: u64,
    ) -> Result<Self> {
        let client = Client::open(redis_url)?;
        
        Ok(Self {
            client,
            queue_name: queue_name.to_string(),
            max_retries,
            retry_delay: Duration::from_millis(retry_delay_ms),
        })
    }

    async fn get_connection(&self) -> Result<redis::aio::Connection> {
        let mut retries = 0;
        loop {
            match self.client.get_async_connection().await {
                Ok(conn) => return Ok(conn),
                Err(e) => {
                    retries += 1;
                    if retries >= self.max_retries {
                        return Err(anyhow::anyhow!("Failed to connect to Redis after {} retries: {}", self.max_retries, e));
                    }
                    eprintln!("Redis connection attempt {} failed: {}, retrying in {:?}", retries, e, self.retry_delay);
                    sleep(self.retry_delay).await;
                }
            }
        }
    }

    async fn execute_with_retry<F, T, Fut>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
        T: Send + 'static,
    {
        let mut retries = 0;
        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    retries += 1;
                    if retries >= self.max_retries {
                        return Err(e);
                    }
                    if let Some(redis_err) = e.downcast_ref::<redis::RedisError>() {
                        if redis_err.is_io_error() || redis_err.is_connection_dropped() {
                            eprintln!("Redis operation failed, retrying: {}", e);
                            sleep(self.retry_delay).await;
                            continue;
                        } else {
                            return Err(e);
                        }
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }

    // Additional utility methods for enhanced functionality
    pub async fn get_queue_info(&self) -> Result<QueueInfo> {
        self.execute_with_retry(|| async {
            let mut conn = self.get_connection().await?;
            let size: usize = conn.llen::<_, usize>(&self.queue_name).await?;
            let ttl: Option<i64> = conn.ttl(&self.queue_name).await?;
            Ok(QueueInfo {
                name: self.queue_name.clone(),
                size,
                ttl_seconds: ttl,
            })
        }).await
    }

    pub async fn set_queue_ttl(&mut self, ttl_seconds: usize) -> Result<()> {
        self.execute_with_retry(|| async {
            let mut conn = self.get_connection().await?;
            conn.expire::<_, ()>(&self.queue_name, ttl_seconds).await?;
            Ok(())
        }).await
    }

    pub async fn health_check(&self) -> Result<RedisHealth> {
        match self.client.get_async_connection().await {
            Ok(mut conn) => {
                let test_key = format!("{}_health_check", self.queue_name);
                let _: () = conn.set::<_, _, ()>(&test_key, "ping").await?;
                let get_result: Option<String> = conn.get(&test_key).await?;
                
                match conn.del::<_, ()>(&test_key).await {
                    Ok(_) => {
                        if get_result.is_some() {
                            Ok(RedisHealth::Healthy)
                        } else {
                            Ok(RedisHealth::Degraded("Set/Get mismatch".to_string()))
                        }
                    }
                    Err(e) => Ok(RedisHealth::Unhealthy(format!("Delete failed: {}", e))),
                }
            }
            Err(e) => Ok(RedisHealth::Unhealthy(format!("Connection failed: {}", e))),
        }
    }

    pub async fn get_memory_usage(&self) -> Result<RedisMemoryInfo> {
        self.execute_with_retry(|| async {
            let mut conn = self.get_connection().await?;
            let info: String = redis::cmd("INFO").query_async(&mut conn).await?;
            Ok(RedisMemoryInfo::from_info(&info))
        }).await
    }
}

#[async_trait::async_trait]
impl Queue for RedisQueue {
    async fn push_url(&mut self, url: &str) -> Result<()> {
        self.execute_with_retry(|| async {
            let mut conn = self.get_connection().await?;
            conn.lpush::<_, _, ()>(&self.queue_name, url).await?;
            Ok(())
        }).await
    }

    async fn pop_url(&mut self) -> Result<Option<String>> {
        self.execute_with_retry(|| async {
            let mut conn = self.get_connection().await?;
            let result: Option<String> = conn.rpop(&self.queue_name, None).await?;
            Ok(result)
        }).await
    }

    async fn get_queue_size(&self) -> Result<usize> {
        self.execute_with_retry(|| async {
            let mut conn = self.get_connection().await?;
            let size: usize = conn.llen::<_, usize>(&self.queue_name).await?;
            Ok(size)
        }).await
    }

    async fn clear_queue(&mut self) -> Result<()> {
        self.execute_with_retry(|| async {
            let mut conn = self.get_connection().await?;
            conn.del::<_, ()>(&self.queue_name).await?;
            Ok(())
        }).await
    }
}

#[derive(Debug, Clone)]
pub struct QueueInfo {
    pub name: String,
    pub size: usize,
    pub ttl_seconds: Option<i64>,
}

#[derive(Debug, Clone)]
pub enum RedisHealth {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

#[derive(Debug, Clone)]
pub struct RedisMemoryInfo {
    pub used_memory: u64,
    pub used_memory_human: String,
    pub used_memory_rss: u64,
    pub used_memory_peak: u64,
    pub total_system_memory: u64
}

impl RedisMemoryInfo {
    pub fn from_info(info: &str) -> Self {
        // Parse Redis INFO command output manually
        let lines: Vec<&str> = info.lines().collect();
        let mut used_memory = 0u64;
        let mut used_memory_human = "0B".to_string();
        let mut used_memory_rss = 0u64;
        let mut used_memory_peak = 0u64;
        let mut total_system_memory = 0u64;
        
        for line in lines {
            if let Some((key, value)) = line.split_once(':') {
                match key {
                    "used_memory" => used_memory = value.parse::<u64>().unwrap_or(0),
                    "used_memory_human" => used_memory_human = value.to_string(),
                    "used_memory_rss" => used_memory_rss = value.parse::<u64>().unwrap_or(0),
                    "used_memory_peak" => used_memory_peak = value.parse::<u64>().unwrap_or(0),
                    "total_system_memory" => total_system_memory = value.parse::<u64>().unwrap_or(0),
                    _ => {}
                }
            }
        }
        
        Self {
            used_memory,
            used_memory_human,
            used_memory_rss,
            used_memory_peak,
            total_system_memory,
        }
    }
}

pub struct RedisSet {
    client: Client,
    set_name: String,
}

impl RedisSet {
    pub async fn new(redis_url: &str, set_name: &str) -> Result<Self> {
        let client = Client::open(redis_url)?;
        
        Ok(Self {
            client,
            set_name: set_name.to_string(),
        })
    }

    async fn get_connection(&self) -> Result<redis::aio::Connection> {
        let conn = self.client.get_async_connection().await?;
        Ok(conn)
    }

    pub async fn add(&mut self, item: &str) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let result: bool = conn.sadd::<_, _, bool>(&self.set_name, item).await?;
        Ok(result)
    }

    pub async fn contains(&self, item: &str) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let result: bool = conn.sismember::<_, _, bool>(&self.set_name, item).await?;
        Ok(result)
    }

    pub async fn get_all(&self) -> Result<Vec<String>> {
        let mut conn = self.get_connection().await?;
        let result: Vec<String> = conn.smembers::<_, Vec<String>>(&self.set_name).await?;
        Ok(result)
    }

    pub async fn size(&self) -> Result<usize> {
        let mut conn = self.get_connection().await?;
        let size: usize = conn.scard::<_, usize>(&self.set_name).await?;
        Ok(size)
    }

    pub async fn clear(&mut self) -> Result<()> {
        let mut conn = self.get_connection().await?;
        conn.del::<_, ()>(&self.set_name).await?;
        Ok(())
    }
}
