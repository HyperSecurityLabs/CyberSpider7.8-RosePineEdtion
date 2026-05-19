use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;
use crate::distributed::{
    DistributedNode, DistributedConfig, NodeMessage, NodeStatus, NodeType, WorkerStatus,
    Task, TaskResult, TaskType, WorkerNode,
};

pub struct Worker {
    config: DistributedConfig,
    capabilities: Vec<TaskType>,
    current_tasks: HashMap<String, Task>,
    max_concurrent_tasks: u32,
    _message_sender: mpsc::UnboundedSender<NodeMessage>,
    message_receiver: Option<mpsc::UnboundedReceiver<NodeMessage>>,
    status: NodeStatus,
    start_time: chrono::DateTime<chrono::Utc>,
    coordinator_address: Option<String>,
}

impl Worker {
    pub fn new(config: DistributedConfig) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        let capabilities = vec![
            TaskType::CrawlUrl,
            TaskType::ProcessContent,
            TaskType::SecurityScan,
        ];

        Self {
            max_concurrent_tasks: config.max_concurrent_tasks,
            capabilities,
            current_tasks: HashMap::new(),
            config: config.clone(),
            _message_sender: sender,
            message_receiver: Some(receiver),
            status: NodeStatus {
                node_id: config.node_id.clone(),
                node_type: NodeType::Worker,
                status: WorkerStatus::Offline,
                uptime_seconds: 0,
                tasks_completed: 0,
                tasks_failed: 0,
                current_tasks: 0,
                last_activity: chrono::Utc::now(),
            },
            start_time: chrono::Utc::now(),
            coordinator_address: None,
        }
    }

    async fn register_with_coordinator(&mut self) -> Result<()> {
        if let Some(coordinator_addr) = &self.config.coordinator_address {
            let worker_node = WorkerNode {
                id: self.config.node_id.clone(),
                name: format!("Worker-{}", self.config.node_id),
                address: self.config.worker_address.clone(),
                status: WorkerStatus::Online,
                capabilities: self.capabilities.clone(),
                current_load: self.current_tasks.len() as u32,
                max_capacity: self.max_concurrent_tasks,
                last_heartbeat: chrono::Utc::now(),
                metadata: HashMap::new(),
            };

            let message = NodeMessage::WorkerRegistration(worker_node);
            self.send_message_to_coordinator(message).await?;
            
            self.coordinator_address = Some(coordinator_addr.clone());
            println!("Registered with coordinator: {}", coordinator_addr);
        }
        
        Ok(())
    }

    async fn send_heartbeat(&self) -> Result<()> {
        if let Some(_) = &self.coordinator_address {
            let message = NodeMessage::Heartbeat;
            self.send_message_to_coordinator(message).await?;
        }
        Ok(())
    }

    async fn send_message_to_coordinator(&self, message: NodeMessage) -> Result<()> {
        if let Some(coordinator_addr) = &self.config.coordinator_address {
            let client = reqwest::Client::new();
            let url = format!("{}/api/message", coordinator_addr);
            
            let response = client
                .post(&url)
                .header("Content-Type", "application/json")
                .header("X-Node-ID", &self.config.node_id)
                .header("X-Node-Type", "worker")
                .json(&message)
                .timeout(Duration::from_secs(10))
                .send()
                .await;
                
            match response {
                Ok(resp) if resp.status().is_success() => {
                    println!("Worker {} message sent to coordinator", self.config.node_id);
                }
                Ok(resp) => {
                    println!("Failed to send message to coordinator: {}", resp.status());
                }
                Err(e) => {
                    println!("Network error sending to coordinator: {}", e);
                }
            }
        }
        Ok(())
    }

    async fn _execute_task(&mut self, task: Task) -> Result<TaskResult> {
        let start_time = std::time::Instant::now();
        let task_id = task.id.clone();
        
        println!("Worker {} executing task: {}", self.config.node_id, task_id);
        
        let result = match task.task_type {
            TaskType::CrawlUrl => self._execute_crawl_task(&task).await,
            TaskType::ProcessContent => self._execute_content_task(&task).await,
            TaskType::SecurityScan => self._execute_security_task(&task).await,
            TaskType::SaveToDatabase => self._execute_database_task(&task).await,
            TaskType::SendNotification => self._execute_notification_task(&task).await,
            TaskType::Custom(ref name) => self._execute_custom_task(&task, name).await,
        };

        let execution_time = start_time.elapsed().as_millis() as u64;
        
        let task_result = TaskResult {
            task_id: task_id.clone(),
            worker_id: self.config.node_id.clone(),
            success: result.is_ok(),
            result: result.as_ref().ok().cloned(),
            error: result.as_ref().err().map(|e| e.to_string()),
            execution_time_ms: execution_time,
            completed_at: chrono::Utc::now(),
        };

        // Remove task from current tasks
        self.current_tasks.remove(&task_id);
        
        // Send result back to coordinator
        let message = NodeMessage::TaskResult(task_result.clone());
        self.send_message_to_coordinator(message).await?;
        
        println!("Worker {} completed task {}: {}", 
                 self.config.node_id, task_id, 
                 if task_result.success { "SUCCESS" } else { "FAILED" });

        Ok(task_result)
    }

    async fn _execute_crawl_task(&self, task: &Task) -> Result<serde_json::Value> {
        let url = task.payload.get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing URL in task payload"))?;

        // Simulate crawling
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        let result = serde_json::json!({
            "url": url,
            "status_code": 200,
            "content_type": "text/html",
            "title": "Example Page",
            "links_found": 15,
            "images_found": 3
        });

        Ok(result)
    }

    async fn _execute_content_task(&self, task: &Task) -> Result<serde_json::Value> {
        let content = task.payload.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing content in task payload"))?;

        // Simulate content processing
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        let word_count = content.split_whitespace().count();
        let char_count = content.chars().count();
        
        let result = serde_json::json!({
            "word_count": word_count,
            "char_count": char_count,
            "processed_at": chrono::Utc::now().to_rfc3339()
        });

        Ok(result)
    }

    async fn _execute_security_task(&self, task: &Task) -> Result<serde_json::Value> {
        let url = task.payload.get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing URL in task payload"))?;

        // Simulate security scanning
        tokio::time::sleep(Duration::from_millis(1000)).await;
        
        let result = serde_json::json!({
            "url": url,
            "vulnerabilities_found": 2,
            "security_score": 7.5,
            "findings": [
                {
                    "type": "XSS",
                    "severity": "medium",
                    "description": "Potential XSS vulnerability"
                },
                {
                    "type": "SQL Injection",
                    "severity": "low",
                    "description": "Possible SQL injection point"
                }
            ]
        });

        Ok(result)
    }

    async fn _execute_database_task(&self, task: &Task) -> Result<serde_json::Value> {
        let _data = task.payload.get("data")
            .ok_or_else(|| anyhow::anyhow!("Missing data in task payload"))?;

        // Simulate database operation
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let result = serde_json::json!({
            "records_affected": 1,
            "operation": "insert",
            "table": "crawl_results",
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        Ok(result)
    }

    async fn _execute_notification_task(&self, task: &Task) -> Result<serde_json::Value> {
        let message = task.payload.get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing message in task payload"))?;

        // Simulate notification sending
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        let result = serde_json::json!({
            "notification_sent": true,
            "message": message,
            "channel": "webhook",
            "sent_at": chrono::Utc::now().to_rfc3339()
        });

        Ok(result)
    }

    async fn _execute_custom_task(&self, task: &Task, task_name: &str) -> Result<serde_json::Value> {
        // Simulate custom task execution
        tokio::time::sleep(Duration::from_millis(300)).await;
        
        let result = serde_json::json!({
            "custom_task": task_name,
            "payload": task.payload,
            "executed_at": chrono::Utc::now().to_rfc3339()
        });

        Ok(result)
    }

    async fn update_status(&mut self) {
        let now = chrono::Utc::now();
        self.status.uptime_seconds = (now - self.start_time).num_seconds() as u64;
        self.status.last_activity = now;
        self.status.current_tasks = self.current_tasks.len() as u32;
        
        // Update worker status based on current load
        if self.current_tasks.len() >= self.max_concurrent_tasks as usize {
            self.status.status = WorkerStatus::Busy;
        } else {
            self.status.status = WorkerStatus::Online;
        }
    }

    async fn _process_messages(&mut self) -> Result<()> {
        if let Some(mut receiver) = self.message_receiver.take() {
            while let Some(message) = receiver.recv().await {
                if let Some(response) = self.handle_message(message).await? {
                    // Send response back to coordinator
                    if let Err(e) = self._message_sender.send(response) {
                        eprintln!("Failed to send response: {}", e);
                    }
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl DistributedNode for Worker {
    async fn start(&mut self) -> Result<()> {
        println!("Starting worker node: {}", self.config.node_id);
        
        // Register with coordinator
        self.register_with_coordinator().await?;
        self.status.status = WorkerStatus::Online;
        
        let mut message_receiver = self.message_receiver.take().unwrap();
        let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(self.config.heartbeat_interval_seconds));
        
        // Main worker loop
        loop {
            tokio::select! {
                Some(message) = message_receiver.recv() => {
                    if let Some(_response) = self.handle_message(message).await? {
                        // Handle response
                    }
                }
                _ = heartbeat_interval.tick() => {
                    self.send_heartbeat().await?;
                    self.update_status().await;
                }
            }
        }
    }

    async fn stop(&mut self) -> Result<()> {
        println!("Stopping worker node: {}", self.config.node_id);
        self.status.status = WorkerStatus::Offline;
        
        // Wait for current tasks to complete or timeout
        let _timeout = Duration::from_secs(self.config.task_timeout_seconds);
        
        while !self.current_tasks.is_empty() {
            tokio::time::sleep(Duration::from_millis(100)).await;
            // In a real implementation, you'd check for timeout and force stop
        }
        
        Ok(())
    }

    async fn get_status(&self) -> NodeStatus {
        self.status.clone()
    }

    async fn handle_message(&mut self, message: NodeMessage) -> Result<Option<NodeMessage>> {
        match message {
            NodeMessage::TaskAssignment(task) => {
                if self.current_tasks.len() < self.max_concurrent_tasks as usize {
                    let task_id = task.id.clone();
                    self.current_tasks.insert(task_id.clone(), task);
                    
                    // Execute task asynchronously
                    let _task_clone = self.current_tasks.get(&task_id).unwrap().clone();
                    let worker_id = self.config.node_id.clone();
                    
                    tokio::spawn(async move {
                        // In a real implementation, this would be handled by the worker's main loop
                        println!("Task {} assigned to worker {}", task_id, worker_id);
                    });
                    
                    self.status.status = WorkerStatus::Busy;
                } else {
                    // Worker is at capacity, reject task
                    return Ok(Some(NodeMessage::TaskResult(TaskResult {
                        task_id: task.id,
                        worker_id: self.config.node_id.clone(),
                        success: false,
                        result: None,
                        error: Some("Worker at capacity".to_string()),
                        execution_time_ms: 0,
                        completed_at: chrono::Utc::now(),
                    })));
                }
                Ok(None)
            }
            NodeMessage::Shutdown => {
                self.stop().await?;
                Ok(None)
            }
            NodeMessage::StatusRequest => {
                Ok(Some(NodeMessage::StatusResponse(self.get_status().await)))
            }
            _ => Ok(None),
        }
    }
}

pub struct WorkerPool {
    workers: HashMap<String, Worker>,
    config: DistributedConfig,
}

impl WorkerPool {
    pub fn new(config: DistributedConfig) -> Self {
        Self {
            workers: HashMap::new(),
            config,
        }
    }

    pub fn add_worker(&mut self, worker_id: String) -> Result<()> {
        let mut worker_config = self.config.clone();
        worker_config.node_id = worker_id.clone();
        
        let worker = Worker::new(worker_config);
        self.workers.insert(worker_id, worker);
        
        Ok(())
    }

    pub async fn start_all_workers(&mut self) -> Result<()> {
        let mut handles = Vec::new();
        
        for (worker_id, _worker) in &mut self.workers {
            let worker_id = worker_id.clone();
            let mut worker_clone = Worker::new(self.config.clone());
            worker_clone.config.node_id = worker_id.clone();
            
            let handle = tokio::spawn(async move {
                worker_clone.start().await
            });
            
            handles.push(handle);
        }
        
        // Wait for all workers (in a real implementation, you'd manage these differently)
        for handle in handles {
            let _ = handle.await;
        }
        
        Ok(())
    }

    pub async fn stop_all_workers(&mut self) -> Result<()> {
        for worker in self.workers.values_mut() {
            worker.stop().await?;
        }
        Ok(())
    }

    pub fn get_worker_count(&self) -> usize {
        self.workers.len()
    }

    pub async fn get_pool_status(&self) -> WorkerPoolStatus {
        let mut total_tasks = 0;
        let mut completed_tasks = 0;
        let mut failed_tasks = 0;
        let mut online_workers = 0;

        for worker in self.workers.values() {
            let status = worker.get_status().await;
            total_tasks += status.tasks_completed + status.tasks_failed;
            completed_tasks += status.tasks_completed;
            failed_tasks += status.tasks_failed;
            
            if status.status == WorkerStatus::Online {
                online_workers += 1;
            }
        }

        WorkerPoolStatus {
            total_workers: self.workers.len(),
            online_workers,
            total_tasks,
            completed_tasks,
            failed_tasks,
            success_rate: if total_tasks > 0 {
                completed_tasks as f64 / total_tasks as f64 * 100.0
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkerPoolStatus {
    pub total_workers: usize,
    pub online_workers: usize,
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub success_rate: f64,
}
