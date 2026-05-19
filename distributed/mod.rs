pub mod coordinator;
pub mod worker;
pub mod http_server;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub task_type: TaskType,
    pub payload: serde_json::Value,
    pub priority: TaskPriority,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub scheduled_at: chrono::DateTime<chrono::Utc>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub timeout_seconds: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    CrawlUrl,
    ProcessContent,
    SecurityScan,
    SaveToDatabase,
    SendNotification,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Ord, PartialOrd, Eq)]
pub enum TaskPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub worker_id: String,
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerNode {
    pub id: String,
    pub name: String,
    pub address: String,
    pub status: WorkerStatus,
    pub capabilities: Vec<TaskType>,
    pub current_load: u32,
    pub max_capacity: u32,
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkerStatus {
    Online,
    Busy,
    Offline,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedConfig {
    pub node_id: String,
    pub node_type: NodeType,
    pub coordinator_address: Option<String>,
    pub worker_address: String,
    pub heartbeat_interval_seconds: u64,
    pub task_timeout_seconds: u64,
    pub max_concurrent_tasks: u32,
    pub enable_auto_scaling: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Coordinator,
    Worker,
    Hybrid,
}

#[async_trait]
pub trait DistributedNode {
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    async fn get_status(&self) -> NodeStatus;
    async fn handle_message(&mut self, message: NodeMessage) -> Result<Option<NodeMessage>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeMessage {
    Heartbeat,
    TaskAssignment(Task),
    TaskResult(TaskResult),
    WorkerRegistration(WorkerNode),
    WorkerUpdate(WorkerNode),
    StatusRequest,
    StatusResponse(NodeStatus),
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub node_id: String,
    pub node_type: NodeType,
    pub status: WorkerStatus,
    pub uptime_seconds: u64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub current_tasks: u32,
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
pub trait TaskQueue: Send + Sync {
    async fn enqueue(&mut self, task: Task) -> Result<()>;
    async fn dequeue(&mut self) -> Result<Option<Task>>;
    async fn peek(&self) -> Result<Option<Task>>;
    async fn size(&self) -> Result<usize>;
    async fn clear(&mut self) -> Result<()>;
    async fn get_tasks_by_priority(&self, priority: TaskPriority) -> Result<Vec<Task>>;
}

#[async_trait]
pub trait WorkerManager {
    async fn register_worker(&mut self, worker: WorkerNode) -> Result<()>;
    async fn unregister_worker(&mut self, worker_id: &str) -> Result<bool>;
    async fn get_available_workers(&self) -> Vec<&WorkerNode>;
    async fn assign_task(&mut self, worker_id: &str, task: Task) -> Result<()>;
    async fn update_worker_status(&mut self, worker_id: &str, status: WorkerStatus) -> Result<()>;
    async fn get_worker_stats(&self) -> WorkerStats;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerStats {
    pub total_workers: usize,
    pub online_workers: usize,
    pub busy_workers: usize,
    pub offline_workers: usize,
    pub total_capacity: u32,
    pub used_capacity: u32,
    pub average_load: f64,
}

pub struct DistributedSpider {
    node: Box<dyn DistributedNode>,
    task_queue: Box<dyn TaskQueue>,
    worker_manager: Box<dyn WorkerManager>,
    config: DistributedConfig,
}

impl DistributedSpider {
    pub fn new(config: DistributedConfig) -> Self {
        // In a real implementation, you would create appropriate implementations
        // based on the node type
        let node: Box<dyn DistributedNode> = match config.node_type {
            NodeType::Coordinator => Box::new(coordinator::Coordinator::new(config.clone())),
            NodeType::Worker => Box::new(worker::Worker::new(config.clone())),
            NodeType::Hybrid => Box::new(coordinator::Coordinator::new(config.clone())),
        };

        Self {
            node,
            task_queue: Box::new(InMemoryTaskQueue::new()),
            worker_manager: Box::new(InMemoryWorkerManager::new()),
            config,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        self.node.start().await
    }

    pub async fn stop(&mut self) -> Result<()> {
        self.node.stop().await
    }

    pub async fn submit_task(&mut self, task: Task) -> Result<()> {
        self.task_queue.enqueue(task).await
    }

    pub async fn get_cluster_status(&self) -> ClusterStatus {
        let worker_stats = self.worker_manager.get_worker_stats().await;
        let queue_size = self.task_queue.size().await.unwrap_or(0);

        ClusterStatus {
            node_id: self.config.node_id.clone(),
            node_type: self.config.node_type.clone(),
            worker_stats,
            queue_size,
            uptime_seconds: 0, // Would be tracked by the node
        }
    }

    pub async fn scale_workers(&mut self, target_count: usize) -> Result<()> {
        let current_workers = self.worker_manager.get_available_workers().await.len();
        
        if current_workers < target_count {
            // Scale up - in a real implementation, this would start new worker processes
            println!("Scaling up from {} to {} workers", current_workers, target_count);
        } else if current_workers > target_count {
            // Scale down
            println!("Scaling down from {} to {} workers", current_workers, target_count);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatus {
    pub node_id: String,
    pub node_type: NodeType,
    pub worker_stats: WorkerStats,
    pub queue_size: usize,
    pub uptime_seconds: u64,
}

pub struct InMemoryTaskQueue {
    tasks: Vec<Task>,
}

impl InMemoryTaskQueue {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }
}

#[async_trait]
impl TaskQueue for InMemoryTaskQueue {
    async fn enqueue(&mut self, task: Task) -> Result<()> {
        self.tasks.push(task);
        self.tasks.sort_by(|a, b| b.priority.cmp(&a.priority).then(a.scheduled_at.cmp(&b.scheduled_at)));
        Ok(())
    }

    async fn dequeue(&mut self) -> Result<Option<Task>> {
        Ok(self.tasks.pop())
    }

    async fn peek(&self) -> Result<Option<Task>> {
        Ok(self.tasks.last().cloned())
    }

    async fn size(&self) -> Result<usize> {
        Ok(self.tasks.len())
    }

    async fn clear(&mut self) -> Result<()> {
        self.tasks.clear();
        Ok(())
    }

    async fn get_tasks_by_priority(&self, priority: TaskPriority) -> Result<Vec<Task>> {
        Ok(self.tasks.iter().filter(|t| t.priority == priority).cloned().collect())
    }
}

pub struct InMemoryWorkerManager {
    workers: HashMap<String, WorkerNode>,
}

impl InMemoryWorkerManager {
    pub fn new() -> Self {
        Self {
            workers: HashMap::new(),
        }
    }
}

#[async_trait]
impl WorkerManager for InMemoryWorkerManager {
    async fn register_worker(&mut self, worker: WorkerNode) -> Result<()> {
        self.workers.insert(worker.id.clone(), worker);
        Ok(())
    }

    async fn unregister_worker(&mut self, worker_id: &str) -> Result<bool> {
        Ok(self.workers.remove(worker_id).is_some())
    }

    async fn get_available_workers(&self) -> Vec<&WorkerNode> {
        self.workers
            .values()
            .filter(|w| w.status == WorkerStatus::Online && w.current_load < w.max_capacity)
            .collect()
    }

    async fn assign_task(&mut self, worker_id: &str, _task: Task) -> Result<()> {
        if let Some(worker) = self.workers.get_mut(worker_id) {
            worker.current_load += 1;
            worker.status = WorkerStatus::Busy;
        }
        Ok(())
    }

    async fn update_worker_status(&mut self, worker_id: &str, status: WorkerStatus) -> Result<()> {
        if let Some(worker) = self.workers.get_mut(worker_id) {
            worker.status = status;
            worker.last_heartbeat = chrono::Utc::now();
        }
        Ok(())
    }

    async fn get_worker_stats(&self) -> WorkerStats {
        let total_workers = self.workers.len();
        let online_workers = self.workers.values().filter(|w| w.status == WorkerStatus::Online).count();
        let busy_workers = self.workers.values().filter(|w| w.status == WorkerStatus::Busy).count();
        let offline_workers = self.workers.values().filter(|w| w.status == WorkerStatus::Offline).count();
        
        let total_capacity: u32 = self.workers.values().map(|w| w.max_capacity).sum();
        let used_capacity: u32 = self.workers.values().map(|w| w.current_load).sum();
        
        let average_load = if total_capacity > 0 {
            used_capacity as f64 / total_capacity as f64 * 100.0
        } else {
            0.0
        };

        WorkerStats {
            total_workers,
            online_workers,
            busy_workers,
            offline_workers,
            total_capacity,
            used_capacity,
            average_load,
        }
    }
}
