use anyhow::Result ;
use async_trait:: async_trait;
use std::collections:: HashMap;
use std::time:: Duration;
use tokio::sync:: mpsc;
use crate::distributed::{
     DistributedNode, DistributedConfig, NodeMessage, NodeStatus, NodeType, WorkerStatus,
     Task, TaskResult, TaskQueue, WorkerNode,
};

pub struct Coordinator {
    config: DistributedConfig,
    workers: HashMap<String, WorkerNode>,
    task_queue: Box<dyn TaskQueue + Send + Sync>,
    _message_sender: mpsc::UnboundedSender<NodeMessage>,
    message_receiver: Option<mpsc::UnboundedReceiver<NodeMessage>>,
    status: NodeStatus,
    start_time: chrono::DateTime<chrono::Utc>,
}

impl Coordinator {
    pub fn new(config: DistributedConfig) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let node_id = config.node_id.clone();
        
        Self {
            config: config.clone(),
            workers: HashMap::new(),
            task_queue: Box::new(crate::distributed::InMemoryTaskQueue::new()),
            _message_sender: sender,
            message_receiver: Some(receiver),
            status: NodeStatus {
                node_id,
                node_type: NodeType::Coordinator,
                status: WorkerStatus::Offline,
                uptime_seconds: 0,
                tasks_completed: 0,
                tasks_failed: 0,
                current_tasks: 0,
                last_activity: chrono::Utc::now(),
            },
            start_time: chrono::Utc::now(),
        }
    }

    async fn _process_messages(&mut self) -> Result<()> {
        if let Some(mut receiver) = self.message_receiver.take() {
            while let Some(message) = receiver.recv().await {
                if let Some(_response) = self.handle_message(message).await? {
                    // Handle response if needed
                }
            }
        }
        Ok(())
    }

    async fn distribute_tasks(&mut self) -> Result<()> {
        loop {
            let task = match self.task_queue.dequeue().await {
                Ok(Some(task)) => task,
                Ok(None) => break,
                Err(e) => return Err(e),
            };
            
            let available_workers = self.get_available_workers();
            
            if let Some(worker) = self.select_best_worker(&available_workers, &task) {
                let worker_id = worker.id.clone();
                self.assign_task_to_worker(worker_id, task).await?;
            } else {
                // No available workers, put task back in queue
                self.task_queue.enqueue(task).await?;
                break;
            }
        }
        
        Ok(())
    }

    fn get_available_workers(&self) -> Vec<&WorkerNode> {
        self.workers
            .values()
            .filter(|w| w.status == WorkerStatus::Online && w.current_load < w.max_capacity)
            .collect()
    }

    fn select_best_worker<'a>(&self, workers: &[&'a WorkerNode], task: &Task) -> Option<&'a WorkerNode> {
        workers
            .iter()
            .filter(|w| w.capabilities.contains(&task.task_type))
            .min_by_key(|w| w.current_load)
            .copied()
    }

    async fn assign_task_to_worker(&mut self, worker_id: String, task: Task) -> Result<()> {
        if let Some(worker) = self.workers.get_mut(&worker_id) {
            worker.current_load += 1;
            worker.status = WorkerStatus::Busy;
            
            // Send task to worker (in a real implementation, this would be network communication)
            let message = NodeMessage::TaskAssignment(task);
            self.send_message_to_worker(&worker_id, message).await?;
        }
        
        Ok(())
    }

    async fn send_message_to_worker(&self, worker_id: &str, message: NodeMessage) -> Result<()> {
        if let Some(worker) = self.workers.get(worker_id) {
            let client = reqwest::Client::new();
            let url = format!("{}/api/message", worker.address);
            
            let response = client
                .post(&url)
                .header("Content-Type", "application/json")
                .header("X-Node-ID", &self.config.node_id)
                .json(&message)
                .timeout(Duration::from_secs(10))
                .send()
                .await;
                
            match response {
                Ok(resp) if resp.status().is_success() => {
                    println!("Message sent successfully to worker: {}", worker_id);
                }
                Ok(resp) => {
                    println!("Failed to send message to worker {}: {}", worker_id, resp.status());
                }
                Err(e) => {
                    println!("Network error sending to worker {}: {}", worker_id, e);
                }
            }
        }
        Ok(())
    }

    async fn check_worker_health(&mut self) -> Result<()> {
        let now = chrono::Utc::now();
        let timeout = chrono::TimeDelta::seconds(self.config.heartbeat_interval_seconds as i64 * 2);
        
        for worker in self.workers.values_mut() {
            if now.signed_duration_since(worker.last_heartbeat) > timeout {
                worker.status = WorkerStatus::Offline;
                println!("Worker {} marked as offline due to missed heartbeat", worker.id);
            }
        }
        
        Ok(())
    }

    async fn update_status(&mut self) {
        let now = chrono::Utc::now();
        self.status.uptime_seconds = (now - self.start_time).num_seconds() as u64;
        self.status.last_activity = now;
        self.status.current_tasks = self.task_queue.size().await.unwrap_or(0) as u32;
    }
}

#[async_trait]
impl DistributedNode for Coordinator {
    async fn start(&mut self) -> Result<()> {
        println!("Starting coordinator node: {}", self.config.node_id);
        self.status.status = WorkerStatus::Online;
        
        let mut message_receiver = self.message_receiver.take().unwrap();
        let _task_queue = Box::new(crate::distributed::InMemoryTaskQueue::new());
        
        // Main coordinator loop
        loop {
            tokio::select! {
                Some(message) = message_receiver.recv() => {
                    if let Some(_response) = self.handle_message(message).await? {
                        // Handle response
                    }
                }
                _ = tokio::time::sleep(Duration::from_secs(1)) => {
                    // Periodic tasks
                    self.check_worker_health().await?;
                    self.distribute_tasks().await?;
                    self.update_status().await;
                }
            }
        }
    }

    async fn stop(&mut self) -> Result<()> {
        println!("Stopping coordinator node: {}", self.config.node_id);
        self.status.status = WorkerStatus::Offline;
        
        // Notify all workers to shutdown
        for worker_id in self.workers.keys() {
            self.send_message_to_worker(worker_id, NodeMessage::Shutdown).await?;
        }
        
        Ok(())
    }

    async fn get_status(&self) -> NodeStatus {
        self.status.clone()
    }

    async fn handle_message(&mut self, message: NodeMessage) -> Result<Option<NodeMessage>> {
        match message {
            NodeMessage::WorkerRegistration(worker) => {
                let worker_id = worker.id.clone();
                self.workers.insert(worker_id.clone(), worker);
                println!("Worker registered: {}", worker_id);
                Ok(None)
            }
            NodeMessage::TaskResult(result) => {
                self.handle_task_result(result).await?;
                Ok(None)
            }
            NodeMessage::Heartbeat => {
                // Update worker heartbeat
                Ok(None)
            }
            NodeMessage::StatusRequest => {
                Ok(Some(NodeMessage::StatusResponse(self.get_status().await)))
            }
            NodeMessage::Shutdown => {
                self.stop().await?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}

impl Coordinator {
    async fn handle_task_result(&mut self, result: TaskResult) -> Result<()> {
        // Update worker load
        if let Some(worker) = self.workers.get_mut(&result.worker_id) {
            worker.current_load = worker.current_load.saturating_sub(1);
            if worker.current_load == 0 {
                worker.status = WorkerStatus::Online;
            }
        }
        
        // Update statistics
        if result.success {
            self.status.tasks_completed += 1;
        } else {
            self.status.tasks_failed += 1;
        }
        
        println!("Task {} completed by worker {}: {}", 
                 result.task_id, result.worker_id, 
                 if result.success { "SUCCESS" } else { "FAILED" });
        
        Ok(())
    }
}

pub struct LoadBalancer {
    strategy: LoadBalancingStrategy,
}

impl LoadBalancer {
    pub fn new(strategy: LoadBalancingStrategy) -> Self {
        Self { strategy }
    }

    pub fn select_worker<'a>(&self, workers: &'a [WorkerNode], task: &Task) -> Option<&'a WorkerNode> {
        let capable_workers: Vec<&WorkerNode> = workers
            .iter()
            .filter(|w| w.status == WorkerStatus::Online 
                    && w.current_load < w.max_capacity 
                    && w.capabilities.contains(&task.task_type))
            .collect();

        if capable_workers.is_empty() {
            return None;
        }

        match self.strategy {
            LoadBalancingStrategy::RoundRobin => {
                let index = rand::random::<usize>() % capable_workers.len();
                capable_workers.get(index).copied()
            }
            LoadBalancingStrategy::LeastLoaded => {
                capable_workers.iter().min_by_key(|w| w.current_load).copied()
            }
            LoadBalancingStrategy::Weighted => {
                capable_workers.iter().max_by(|a, b| {
                    let a_score = (1.0 - (a.current_load as f64 / a.max_capacity as f64)) * a.max_capacity as f64;
                    let b_score = (1.0 - (b.current_load as f64 / b.max_capacity as f64)) * b.max_capacity as f64;
                    a_score.partial_cmp(&b_score).unwrap_or(std::cmp::Ordering::Equal)
                }).copied()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastLoaded,
    Weighted,
}

pub struct ClusterManager {
    coordinator: Coordinator,
    auto_scaling_enabled: bool,
    min_workers: usize,
    max_workers: usize,
    scale_up_threshold: f64,
    scale_down_threshold: f64,
}

impl ClusterManager {
    pub fn new(coordinator: Coordinator) -> Self {
        Self {
            coordinator,
            auto_scaling_enabled: false,
            min_workers: 1,
            max_workers: 10,
            scale_up_threshold: 0.8,
            scale_down_threshold: 0.2,
        }
    }

    pub fn with_auto_scaling(mut self, enabled: bool) -> Self {
        self.auto_scaling_enabled = enabled;
        self
    }

    pub fn with_worker_limits(mut self, min: usize, max: usize) -> Self {
        self.min_workers = min;
        self.max_workers = max;
        self
    }

    pub async fn start(&mut self) -> Result<()> {
        if self.auto_scaling_enabled {
            self.start_auto_scaling().await?;
        }
        
        self.coordinator.start().await
    }

    async fn start_auto_scaling(&mut self) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            self.check_scaling_needed().await?;
        }
    }

    async fn check_scaling_needed(&mut self) -> Result<()> {
        let current_workers = self.coordinator.workers.len();
        let queue_size = self.coordinator.task_queue.size().await?;
        
        let avg_load = if current_workers > 0 {
            let total_load: u32 = self.coordinator.workers.values().map(|w| w.current_load).sum();
            total_load as f64 / current_workers as f64
        } else {
            0.0
        };

        // Scale up if needed
        if (avg_load > self.scale_up_threshold || queue_size > 10) 
            && current_workers < self.max_workers {
            self.scale_up().await?;
        }
        
        // Scale down if needed
        else if avg_load < self.scale_down_threshold 
            && queue_size == 0 
            && current_workers > self.min_workers {
            self.scale_down().await?;
        }

        Ok(())
    }

    async fn scale_up(&mut self) -> Result<()> {
        println!("Scaling up: adding new worker");
        // In a real implementation, this would start a new worker process
        Ok(())
    }

    async fn scale_down(&mut self) -> Result<()> {
        println!("Scaling down: removing idle worker");
        // In a real implementation, this would gracefully shutdown a worker
        Ok(())
    }

    pub async fn get_cluster_metrics(&self) -> ClusterMetrics {
        let worker_count = self.coordinator.workers.len();
        let queue_size = self.coordinator.task_queue.size().await.unwrap_or(0);
        let online_workers = self.coordinator.workers.values()
            .filter(|w| w.status == WorkerStatus::Online)
            .count();
        
        let total_capacity: u32 = self.coordinator.workers.values().map(|w| w.max_capacity).sum();
        let used_capacity: u32 = self.coordinator.workers.values().map(|w| w.current_load).sum();
        
        ClusterMetrics {
            worker_count,
            online_workers,
            queue_size,
            total_capacity,
            used_capacity,
            utilization_rate: if total_capacity > 0 {
                used_capacity as f64 / total_capacity as f64 * 100.0
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClusterMetrics {
    pub worker_count: usize,
    pub online_workers: usize,
    pub queue_size: usize,
    pub total_capacity: u32,
    pub used_capacity: u32,
    pub utilization_rate: f64,
}
