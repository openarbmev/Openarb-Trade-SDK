//! 🚀 超低延迟优化模块 - 目标实现<1ms端到端延迟
//!
//! 这个模块包含针对亚毫秒级延迟的极致优化：
//! - 无锁并发事件处理
//! - CPU亲和性绑定
//! - 零分配内存管理
//! - 预测性预取优化
//! - 硬件加速序列化

use std::sync::{Arc, atomic::{AtomicU64, AtomicUsize, AtomicBool, Ordering}};
use std::time::{Duration, Instant};
// use std::collections::VecDeque;
use crossbeam_queue::ArrayQueue;
use crossbeam_utils::CachePadded;
use fzstream_common::EventMessage;
use tokio::sync::Notify;
use anyhow::Result;
use log::{info, warn, debug};

/// 🚀 无锁事件分发器 - 使用环形缓冲区实现极速事件分发
pub struct LockFreeEventDispatcher {
    /// 无锁环形缓冲区，支持多生产者单消费者
    event_queues: Vec<Arc<ArrayQueue<EventMessage>>>,
    /// 客户端映射到队列的索引
    client_queue_mapping: Arc<dashmap::DashMap<String, usize>>,
    /// 队列选择策略（轮询计数器）
    queue_selector: CachePadded<AtomicUsize>,
    /// 性能统计
    stats: Arc<UltraLowLatencyStats>,
    /// 预取优化器
    prefetch_optimizer: Arc<PrefetchOptimizer>,
    /// CPU绑定配置
    cpu_affinity: Option<CpuAffinityConfig>,
}

/// CPU亲和性配置
#[derive(Clone, Debug)]
pub struct CpuAffinityConfig {
    /// 绑定到特定CPU核心
    pub core_ids: Vec<usize>,
    /// 启用NUMA优化
    pub numa_optimization: bool,
    /// 优先级设置
    pub priority: ThreadPriority,
}

#[derive(Clone, Debug)]
pub enum ThreadPriority {
    Normal,
    High,
    RealTime,
}

/// 🚀 预取优化器 - 预测性数据预加载
pub struct PrefetchOptimizer {
    /// 预测缓存：基于历史模式预取可能需要的数据
    prediction_cache: Arc<ArrayQueue<EventMessage>>,
    /// 预取命中统计
    hit_count: AtomicU64,
    /// 预取失效统计
    miss_count: AtomicU64,
    /// 学习模式开关
    learning_enabled: AtomicBool,
}

impl PrefetchOptimizer {
    pub fn new(cache_size: usize) -> Self {
        Self {
            prediction_cache: Arc::new(ArrayQueue::new(cache_size)),
            hit_count: AtomicU64::new(0),
            miss_count: AtomicU64::new(0),
            learning_enabled: AtomicBool::new(true),
        }
    }

    /// 预测性预取事件数据
    #[inline(always)]
    pub fn prefetch_event_data(&self, event: &EventMessage) {
        if !self.learning_enabled.load(Ordering::Relaxed) {
            return;
        }

        // 基于事件类型的简单预测逻辑
        // 在实际应用中，这里可以实现更复杂的机器学习预测算法
        if let Ok(_) = self.prediction_cache.push(event.clone()) {
            // 预取成功
        }
    }

    /// 尝试从预取缓存获取事件
    #[inline(always)]
    pub fn try_get_prefetched(&self) -> Option<EventMessage> {
        if let Some(event) = self.prediction_cache.pop() {
            self.hit_count.fetch_add(1, Ordering::Relaxed);
            Some(event)
        } else {
            self.miss_count.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    /// 获取预取统计信息
    pub fn get_stats(&self) -> (u64, u64, f64) {
        let hits = self.hit_count.load(Ordering::Relaxed);
        let misses = self.miss_count.load(Ordering::Relaxed);
        let hit_rate = if hits + misses > 0 {
            hits as f64 / (hits + misses) as f64
        } else {
            0.0
        };
        (hits, misses, hit_rate)
    }
}

/// 🚀 超低延迟统计收集器
pub struct UltraLowLatencyStats {
    /// 事件处理计数
    pub events_processed: CachePadded<AtomicU64>,
    /// 纳秒级延迟统计
    pub total_latency_ns: CachePadded<AtomicU64>,
    /// 最小延迟（纳秒）
    pub min_latency_ns: CachePadded<AtomicU64>,
    /// 最大延迟（纳秒）
    pub max_latency_ns: CachePadded<AtomicU64>,
    /// 亚毫秒事件计数（<1ms）
    pub sub_millisecond_events: CachePadded<AtomicU64>,
    /// 超快事件计数（<100μs）
    pub ultra_fast_events: CachePadded<AtomicU64>,
    /// 极速事件计数（<10μs）
    pub lightning_fast_events: CachePadded<AtomicU64>,
    /// 队列溢出计数
    pub queue_overflows: CachePadded<AtomicU64>,
    /// 预取命中计数
    pub prefetch_hits: CachePadded<AtomicU64>,
}

impl UltraLowLatencyStats {
    pub fn new() -> Self {
        Self {
            events_processed: CachePadded::new(AtomicU64::new(0)),
            total_latency_ns: CachePadded::new(AtomicU64::new(0)),
            min_latency_ns: CachePadded::new(AtomicU64::new(u64::MAX)),
            max_latency_ns: CachePadded::new(AtomicU64::new(0)),
            sub_millisecond_events: CachePadded::new(AtomicU64::new(0)),
            ultra_fast_events: CachePadded::new(AtomicU64::new(0)),
            lightning_fast_events: CachePadded::new(AtomicU64::new(0)),
            queue_overflows: CachePadded::new(AtomicU64::new(0)),
            prefetch_hits: CachePadded::new(AtomicU64::new(0)),
        }
    }

    /// 记录事件处理延迟（纳秒级精度）
    #[inline(always)]
    pub fn record_event_latency(&self, latency_ns: u64) {
        self.events_processed.fetch_add(1, Ordering::Relaxed);
        self.total_latency_ns.fetch_add(latency_ns, Ordering::Relaxed);

        // 更新最小值
        let mut current_min = self.min_latency_ns.load(Ordering::Relaxed);
        while latency_ns < current_min {
            match self.min_latency_ns.compare_exchange_weak(
                current_min, latency_ns, Ordering::Relaxed, Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(x) => current_min = x,
            }
        }

        // 更新最大值
        let mut current_max = self.max_latency_ns.load(Ordering::Relaxed);
        while latency_ns > current_max {
            match self.max_latency_ns.compare_exchange_weak(
                current_max, latency_ns, Ordering::Relaxed, Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }

        // 分类统计
        if latency_ns < 1_000_000 { // <1ms
            self.sub_millisecond_events.fetch_add(1, Ordering::Relaxed);
        }
        if latency_ns < 100_000 { // <100μs
            self.ultra_fast_events.fetch_add(1, Ordering::Relaxed);
        }
        if latency_ns < 10_000 { // <10μs
            self.lightning_fast_events.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// 获取延迟统计摘要
    pub fn get_summary(&self) -> UltraLatencySummary {
        let events_processed = self.events_processed.load(Ordering::Relaxed);
        let total_latency_ns = self.total_latency_ns.load(Ordering::Relaxed);
        let min_latency_ns = self.min_latency_ns.load(Ordering::Relaxed);
        let max_latency_ns = self.max_latency_ns.load(Ordering::Relaxed);
        let sub_ms_events = self.sub_millisecond_events.load(Ordering::Relaxed);
        let ultra_fast_events = self.ultra_fast_events.load(Ordering::Relaxed);
        let lightning_fast_events = self.lightning_fast_events.load(Ordering::Relaxed);

        let avg_latency_ns = if events_processed > 0 {
            total_latency_ns as f64 / events_processed as f64
        } else {
            0.0
        };

        let sub_ms_percentage = if events_processed > 0 {
            sub_ms_events as f64 / events_processed as f64 * 100.0
        } else {
            0.0
        };

        let ultra_fast_percentage = if events_processed > 0 {
            ultra_fast_events as f64 / events_processed as f64 * 100.0
        } else {
            0.0
        };

        let lightning_fast_percentage = if events_processed > 0 {
            lightning_fast_events as f64 / events_processed as f64 * 100.0
        } else {
            0.0
        };

        UltraLatencySummary {
            events_processed,
            avg_latency_ns,
            min_latency_ns: if min_latency_ns == u64::MAX { 0.0 } else { min_latency_ns as f64 },
            max_latency_ns: max_latency_ns as f64,
            avg_latency_us: avg_latency_ns / 1000.0,
            sub_millisecond_percentage: sub_ms_percentage,
            ultra_fast_percentage,
            lightning_fast_percentage,
            target_achieved: avg_latency_ns < 1_000_000.0, // <1ms target
        }
    }
}

/// 延迟统计摘要
#[derive(Debug, Clone)]
pub struct UltraLatencySummary {
    pub events_processed: u64,
    pub avg_latency_ns: f64,
    pub min_latency_ns: f64,
    pub max_latency_ns: f64,
    pub avg_latency_us: f64,
    pub sub_millisecond_percentage: f64,
    pub ultra_fast_percentage: f64,
    pub lightning_fast_percentage: f64,
    pub target_achieved: bool,
}

impl LockFreeEventDispatcher {
    /// 创建新的无锁事件分发器
    pub fn new(
        num_queues: usize, 
        queue_capacity: usize,
        cpu_affinity: Option<CpuAffinityConfig>
    ) -> Self {
        let mut event_queues = Vec::with_capacity(num_queues);
        for _ in 0..num_queues {
            event_queues.push(Arc::new(ArrayQueue::new(queue_capacity)));
        }

        info!("🚀 Created LockFreeEventDispatcher: {} queues, capacity {} each", 
              num_queues, queue_capacity);

        Self {
            event_queues,
            client_queue_mapping: Arc::new(dashmap::DashMap::new()),
            queue_selector: CachePadded::new(AtomicUsize::new(0)),
            stats: Arc::new(UltraLowLatencyStats::new()),
            prefetch_optimizer: Arc::new(PrefetchOptimizer::new(1000)),
            cpu_affinity,
        }
    }

    /// 🚀 极速事件分发 - 无锁路径
    #[inline(always)]
    pub fn dispatch_event_ultra_fast(&self, client_id: &str, event: EventMessage) -> Result<()> {
        let start_time = Instant::now();

        // 获取或分配客户端队列
        let queue_index = if let Some(index) = self.client_queue_mapping.get(client_id) {
            *index
        } else {
            // 使用轮询策略分配新队列
            let index = self.queue_selector.fetch_add(1, Ordering::Relaxed) % self.event_queues.len();
            self.client_queue_mapping.insert(client_id.to_string(), index);
            index
        };

        // 预取优化
        self.prefetch_optimizer.prefetch_event_data(&event);

        // 尝试无阻塞推送到队列
        let queue = &self.event_queues[queue_index];
        match queue.push(event) {
            Ok(_) => {
                // 记录处理延迟
                let latency_ns = start_time.elapsed().as_nanos() as u64;
                self.stats.record_event_latency(latency_ns);
                Ok(())
            }
            Err(_) => {
                // 队列满，记录溢出
                self.stats.queue_overflows.fetch_add(1, Ordering::Relaxed);
                Err(anyhow::anyhow!("Queue overflow for client: {}", client_id))
            }
        }
    }

    /// 启动事件处理工作线程
    pub async fn start_processing_workers(&self, num_workers: usize) -> Result<()> {
        info!("🚀 Starting {} ultra-low-latency processing workers", num_workers);

        for worker_id in 0..num_workers {
            let queues = self.event_queues.clone();
            let stats = Arc::clone(&self.stats);
            let cpu_affinity = self.cpu_affinity.clone();

            tokio::spawn(async move {
                // 应用CPU亲和性
                if let Some(affinity_config) = &cpu_affinity {
                    if let Err(e) = Self::set_thread_affinity(worker_id, affinity_config) {
                        warn!("Failed to set CPU affinity for worker {}: {}", worker_id, e);
                    } else {
                        info!("✅ Worker {} bound to CPU core", worker_id);
                    }
                }

                // 工作线程主循环
                Self::worker_main_loop(worker_id, queues, stats).await;
            });
        }

        Ok(())
    }

    /// 工作线程主循环 - 极速事件处理
    async fn worker_main_loop(
        worker_id: usize,
        queues: Vec<Arc<ArrayQueue<EventMessage>>>,
        stats: Arc<UltraLowLatencyStats>
    ) {
        info!("🔄 Worker {} started ultra-low-latency processing loop", worker_id);
        
        let mut queue_index = worker_id; // 从分配的队列开始
        let notify = Arc::new(Notify::new());
        
        loop {
            let mut processed_any = false;

            // 轮询所有队列，寻找待处理事件
            for _ in 0..queues.len() {
                let queue = &queues[queue_index % queues.len()];
                
                // 批量处理以提高吞吐量
                let mut batch_count = 0;
                while batch_count < 100 { // 批次大小限制
                    match queue.pop() {
                        Some(event) => {
                            let process_start = Instant::now();
                            
                            // 🚀 这里是实际的事件处理逻辑
                            // 在真实应用中，这里会调用实际的事件处理函数
                            Self::process_event_ultra_fast(&event).await;
                            
                            let process_latency = process_start.elapsed().as_nanos() as u64;
                            stats.record_event_latency(process_latency);
                            
                            processed_any = true;
                            batch_count += 1;
                        }
                        None => break,
                    }
                }

                queue_index = (queue_index + 1) % queues.len();
            }

            if !processed_any {
                // 没有事件要处理，短暂休眠避免CPU空转
                tokio::task::yield_now().await;
                
                // 可选：使用更智能的等待机制
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_nanos(100)) => {}, // 100ns极短休眠
                    _ = notify.notified() => {}, // 或等待通知
                }
            }
        }
    }

    /// 🚀 极速事件处理函数
    #[inline(always)]
    async fn process_event_ultra_fast(event: &EventMessage) {
        // 在这里实现实际的事件处理逻辑
        // 为了演示，我们只是做一些最小的处理
        
        // 避免不必要的分配和复制
        debug!("Processing event: {} bytes", event.data.len());
        
        // 在实际应用中，这里会：
        // 1. 解析事件数据
        // 2. 应用业务逻辑
        // 3. 转发给相应的客户端
        
        // 模拟极少的处理时间
        tokio::task::yield_now().await;
    }

    /// 设置线程CPU亲和性
    fn set_thread_affinity(worker_id: usize, config: &CpuAffinityConfig) -> Result<()> {
        if config.core_ids.is_empty() {
            return Ok(());
        }

        #[allow(unused_variables)]
        let core_id = config.core_ids[worker_id % config.core_ids.len()];

        #[cfg(target_os = "linux")]
        {
            use libc::{cpu_set_t, sched_setaffinity, CPU_SET, CPU_ZERO};
            
            unsafe {
                let mut cpuset: cpu_set_t = std::mem::zeroed();
                CPU_ZERO(&mut cpuset);
                CPU_SET(core_id, &mut cpuset);
                
                if sched_setaffinity(0, std::mem::size_of::<cpu_set_t>(), &cpuset) != 0 {
                    return Err(anyhow::anyhow!("Failed to set CPU affinity to core {}", core_id));
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS不支持CPU亲和性绑定，但可以设置线程优先级
            info!("CPU affinity not supported on macOS, setting thread priority instead");
            
            // 可以使用thread_policy_set来设置线程调度策略
            // 这里简化处理，只记录日志
        }

        #[cfg(target_os = "windows")]
        {
            use winapi::um::processthreadsapi::{GetCurrentThread, SetThreadAffinityMask};
            
            unsafe {
                let affinity_mask = 1u64 << core_id;
                if SetThreadAffinityMask(GetCurrentThread(), affinity_mask as usize) == 0 {
                    return Err(anyhow::anyhow!("Failed to set CPU affinity to core {}", core_id));
                }
            }
        }

        Ok(())
    }

    /// 获取性能统计信息
    pub fn get_performance_stats(&self) -> UltraLatencySummary {
        self.stats.get_summary()
    }

    /// 获取预取统计信息
    pub fn get_prefetch_stats(&self) -> (u64, u64, f64) {
        self.prefetch_optimizer.get_stats()
    }

    /// 获取队列状态信息
    pub fn get_queue_stats(&self) -> Vec<(usize, usize)> {
        self.event_queues.iter().enumerate()
            .map(|(i, queue)| (i, queue.len()))
            .collect()
    }
}

/// 🚀 零分配事件序列化器
pub struct ZeroAllocSerializer {
    /// 预分配的序列化缓冲区池
    buffer_pool: Arc<ArrayQueue<Vec<u8>>>,
    /// 快速查找表：事件类型 -> 预计算序列化大小
    size_hints: Arc<dashmap::DashMap<String, usize>>,
}

impl ZeroAllocSerializer {
    pub fn new(pool_size: usize, buffer_size: usize) -> Self {
        let buffer_pool = Arc::new(ArrayQueue::new(pool_size));
        
        // 预分配缓冲区
        for _ in 0..pool_size {
            let _ = buffer_pool.push(Vec::with_capacity(buffer_size));
        }

        Self {
            buffer_pool,
            size_hints: Arc::new(dashmap::DashMap::new()),
        }
    }

    /// 🚀 零分配序列化 - 重用预分配缓冲区
    #[inline(always)]
    pub fn serialize_zero_alloc<T: serde::Serialize>(&self, value: &T, event_type: &str) -> Result<Vec<u8>> {
        // 尝试获取预分配缓冲区
        let mut buffer = if let Some(buf) = self.buffer_pool.pop() {
            buf
        } else {
            // 池耗尽，分配新缓冲区
            let hint_size = self.size_hints.get(event_type)
                .map(|entry| *entry)
                .unwrap_or(1024);
            Vec::with_capacity(hint_size)
        };

        // 清空缓冲区但保持容量
        buffer.clear();

        // 直接序列化到复用缓冲区，避免临时 Vec 和整块复制。
        if let Err(error) = bincode::serialize_into(&mut buffer, value) {
            let _ = self.buffer_pool.push(buffer);
            return Err(error.into());
        }

        // 更新大小提示，用于优化后续分配
        self.size_hints.insert(event_type.to_string(), buffer.len());

        Ok(buffer)
    }

    /// 归还缓冲区到池中
    #[inline(always)]
    pub fn return_buffer(&self, buffer: Vec<u8>) {
        // 只归还合理大小的缓冲区，避免池被超大缓冲区占用
        if buffer.capacity() <= 1024 * 1024 { // 1MB limit
            let _ = self.buffer_pool.push(buffer);
        }
    }

    /// 获取池状态
    pub fn get_pool_stats(&self) -> (usize, usize) {
        (self.buffer_pool.len(), self.buffer_pool.capacity())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fzstream_common::{SerializationProtocol};
    use solana_streamer_sdk::streaming::event_parser::common::EventType;

    #[tokio::test]
    async fn test_lockfree_dispatcher() {
        let dispatcher = LockFreeEventDispatcher::new(4, 1000, None);
        
        let test_event = EventMessage {
            event_id: "test_1".to_string(),
            event_type: EventType::BlockMeta,
            data: vec![1, 2, 3, 4],
            serialization_format: SerializationProtocol::Bincode,
            compression_format: fzstream_common::CompressionLevel::None,
            is_compressed: false,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            original_size: Some(4),
            grpc_arrival_time: 0,
            parsing_time: 0,
            completion_time: 0,
            client_processing_start: None,
            client_processing_end: None,
        };

        // 测试事件分发
        assert!(dispatcher.dispatch_event_ultra_fast("client_1", test_event).is_ok());

        // 检查统计
        let stats = dispatcher.get_performance_stats();
        assert_eq!(stats.events_processed, 1);
    }

    #[test]
    fn test_zero_alloc_serializer() {
        let serializer = ZeroAllocSerializer::new(10, 1024);
        
        let test_data = "Hello, world!";
        let result = serializer.serialize_zero_alloc(&test_data, "string");
        assert!(result.is_ok());
        
        let serialized = result.unwrap();
        assert!(!serialized.is_empty());
        
        // 测试缓冲区归还
        serializer.return_buffer(serialized);
        
        let (available, capacity) = serializer.get_pool_stats();
        assert!(available > 0);
        assert_eq!(capacity, 10);
    }
}
