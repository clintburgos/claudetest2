# Error Handling & Recovery System Design

## Overview

A comprehensive error handling system that ensures graceful degradation, data integrity, and smooth recovery from failures. The system covers everything from runtime panics to save file corruption, network errors, and resource exhaustion.

## Error Categories

### 1. Critical Errors (Unrecoverable)
- Memory allocation failures
- Core system initialization failures
- Graphics device lost
- Corrupted executable

### 2. Severe Errors (Require Restart)
- Save system corruption beyond repair
- Audio device failures
- Critical thread panics
- Stack overflow

### 3. Major Errors (Partial System Failure)
- Subsystem initialization failures
- Resource loading failures
- Network disconnections
- Database corruption

### 4. Minor Errors (Recoverable)
- Asset loading failures
- Temporary file I/O errors
- Network timeouts
- Invalid user input

### 5. Warnings (Non-Critical)
- Performance degradation
- Memory pressure
- Deprecated feature usage
- Non-optimal settings

## Core Architecture

```rust
use std::panic;
use std::sync::Arc;
use std::sync::Mutex;

// Central error type
#[derive(Debug, Clone)]
pub enum GameError {
    // System errors
    SystemError {
        subsystem: SubsystemType,
        severity: ErrorSeverity,
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    // IO errors
    IoError {
        operation: IoOperation,
        path: Option<PathBuf>,
        reason: String,
    },
    
    // Network errors
    NetworkError {
        operation: NetworkOperation,
        endpoint: Option<String>,
        retry_count: u32,
    },
    
    // Game logic errors
    GameLogicError {
        system: GameSystem,
        state: String,
        recovery_action: RecoveryAction,
    },
    
    // Resource errors
    ResourceError {
        resource_type: ResourceType,
        identifier: String,
        fallback: Option<String>,
    },
    
    // Save/Load errors
    SaveError {
        save_type: SaveType,
        corruption_level: CorruptionLevel,
        backup_available: bool,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorSeverity {
    Critical,   // Requires immediate shutdown
    Severe,     // Requires restart
    Major,      // Degraded functionality
    Minor,      // Recoverable
    Warning,    // Informational
}

#[derive(Debug, Clone, Copy)]
pub enum RecoveryAction {
    None,
    Retry,
    Fallback,
    Reset,
    Disable,
    RestoreBackup,
}

// Global error handler
pub struct ErrorHandler {
    error_log: Arc<Mutex<ErrorLog>>,
    recovery_strategies: HashMap<GameError, RecoveryStrategy>,
    panic_handler: Option<Box<dyn Fn(&panic::PanicInfo) + Send + Sync>>,
    error_reporters: Vec<Box<dyn ErrorReporter>>,
}

pub struct ErrorLog {
    entries: VecDeque<ErrorEntry>,
    max_entries: usize,
    persistent_log: Option<File>,
    error_counts: HashMap<String, u32>,
}

pub struct ErrorEntry {
    timestamp: Instant,
    error: GameError,
    context: ErrorContext,
    stack_trace: Option<String>,
    recovery_attempted: bool,
    recovery_result: Option<Result<(), String>>,
}
```

### Error Context System

```rust
pub struct ErrorContext {
    pub game_state: GameStateSnapshot,
    pub system_info: SystemInfo,
    pub performance_metrics: PerformanceSnapshot,
    pub user_actions: VecDeque<UserAction>,
}

pub struct GameStateSnapshot {
    pub world_seed: u64,
    pub creature_count: usize,
    pub simulation_time: f64,
    pub active_systems: Vec<String>,
    pub memory_usage: MemoryStats,
}

impl ErrorHandler {
    pub fn capture_context(&self) -> ErrorContext {
        ErrorContext {
            game_state: self.snapshot_game_state(),
            system_info: self.get_system_info(),
            performance_metrics: self.get_performance_snapshot(),
            user_actions: self.get_recent_user_actions(10),
        }
    }
    
    pub fn handle_error(&mut self, error: GameError) -> Result<(), GameError> {
        let context = self.capture_context();
        let entry = ErrorEntry {
            timestamp: Instant::now(),
            error: error.clone(),
            context,
            stack_trace: self.capture_stack_trace(),
            recovery_attempted: false,
            recovery_result: None,
        };
        
        // Log error
        self.log_error(&entry);
        
        // Determine recovery strategy
        match self.get_recovery_strategy(&error) {
            Some(strategy) => {
                let result = self.attempt_recovery(strategy, &error);
                entry.recovery_attempted = true;
                entry.recovery_result = Some(result.clone());
                result
            }
            None => Err(error),
        }
    }
}
```

### Panic Handling

```rust
pub struct PanicHandler {
    crash_dumps_path: PathBuf,
    auto_restart: bool,
    restart_limit: u32,
    restart_count: Arc<Mutex<u32>>,
}

impl PanicHandler {
    pub fn install(self: Arc<Self>) {
        let handler = self.clone();
        panic::set_hook(Box::new(move |panic_info| {
            handler.handle_panic(panic_info);
        }));
    }
    
    fn handle_panic(&self, panic_info: &panic::PanicInfo) {
        // Create crash dump
        let crash_dump = CrashDump {
            timestamp: Utc::now(),
            panic_info: format!("{}", panic_info),
            thread_name: thread::current().name().map(String::from),
            backtrace: Backtrace::capture(),
            game_state: self.capture_game_state(),
            system_info: self.get_system_info(),
        };
        
        // Save crash dump
        let dump_path = self.save_crash_dump(&crash_dump);
        
        // Attempt auto-recovery
        if self.should_attempt_restart() {
            self.attempt_restart(dump_path);
        } else {
            self.show_crash_dialog(&crash_dump);
        }
    }
    
    fn save_crash_dump(&self, dump: &CrashDump) -> PathBuf {
        let filename = format!("crash_{}.json", dump.timestamp.format("%Y%m%d_%H%M%S"));
        let path = self.crash_dumps_path.join(filename);
        
        if let Ok(file) = File::create(&path) {
            serde_json::to_writer_pretty(file, dump).ok();
        }
        
        // Also save a human-readable version
        let txt_path = path.with_extension("txt");
        if let Ok(mut file) = File::create(&txt_path) {
            writeln!(file, "Creature Simulation Crash Report").ok();
            writeln!(file, "================================").ok();
            writeln!(file, "Time: {}", dump.timestamp).ok();
            writeln!(file, "Thread: {:?}", dump.thread_name).ok();
            writeln!(file, "\nPanic Info:\n{}", dump.panic_info).ok();
            writeln!(file, "\nBacktrace:\n{:?}", dump.backtrace).ok();
        }
        
        path
    }
}
```

### Save System Error Recovery

```rust
pub struct SaveErrorRecovery {
    backup_manager: BackupManager,
    corruption_detector: CorruptionDetector,
    repair_strategies: HashMap<CorruptionType, RepairStrategy>,
}

pub struct BackupManager {
    backup_dir: PathBuf,
    max_backups: usize,
    backup_interval: Duration,
    rolling_backups: VecDeque<BackupInfo>,
}

pub struct CorruptionDetector {
    checksum_verifier: ChecksumVerifier,
    structure_validator: StructureValidator,
    data_bounds_checker: DataBoundsChecker,
}

impl SaveErrorRecovery {
    pub fn handle_save_corruption(
        &mut self,
        save_path: &Path,
        error: SaveError,
    ) -> Result<SaveData, SaveError> {
        // First, try to detect the type and extent of corruption
        let corruption_report = self.corruption_detector.analyze(save_path)?;
        
        match corruption_report.severity {
            CorruptionSeverity::Minor => {
                // Attempt repair
                self.attempt_repair(save_path, &corruption_report)
            }
            CorruptionSeverity::Major => {
                // Try to salvage what we can
                self.salvage_save_data(save_path, &corruption_report)
            }
            CorruptionSeverity::Critical => {
                // Restore from backup
                self.restore_from_backup(save_path)
            }
        }
    }
    
    fn attempt_repair(
        &self,
        save_path: &Path,
        report: &CorruptionReport,
    ) -> Result<SaveData, SaveError> {
        let mut save_data = self.load_raw_save(save_path)?;
        
        for corruption in &report.corruptions {
            if let Some(strategy) = self.repair_strategies.get(&corruption.corruption_type) {
                match strategy {
                    RepairStrategy::RebuildIndex => {
                        save_data.rebuild_indices();
                    }
                    RepairStrategy::RestoreDefaults(defaults) => {
                        save_data.apply_defaults(defaults);
                    }
                    RepairStrategy::RemoveInvalid => {
                        save_data.remove_invalid_entries();
                    }
                    RepairStrategy::RecalculateChecksums => {
                        save_data.recalculate_all_checksums();
                    }
                }
            }
        }
        
        // Verify repair
        if self.corruption_detector.verify(&save_data).is_ok() {
            Ok(save_data)
        } else {
            Err(SaveError::RepairFailed)
        }
    }
    
    fn salvage_save_data(
        &self,
        save_path: &Path,
        report: &CorruptionReport,
    ) -> Result<SaveData, SaveError> {
        let raw_data = fs::read(save_path)?;
        let mut salvaged = SaveData::new();
        
        // Try to extract valid chunks
        for chunk in self.extract_chunks(&raw_data) {
            if let Ok(partial_data) = self.parse_chunk(&chunk) {
                salvaged.merge_partial(partial_data);
            }
        }
        
        // Fill in missing critical data
        salvaged.fill_missing_critical_data();
        
        if salvaged.is_playable() {
            Ok(salvaged)
        } else {
            self.restore_from_backup(save_path)
        }
    }
}
```

### Network Error Handling

```rust
pub struct NetworkErrorHandler {
    retry_policies: HashMap<NetworkOperation, RetryPolicy>,
    circuit_breakers: HashMap<String, CircuitBreaker>,
    fallback_handlers: HashMap<NetworkOperation, FallbackHandler>,
}

pub struct RetryPolicy {
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
    backoff_factor: f32,
    jitter: bool,
}

pub struct CircuitBreaker {
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
    state: CircuitState,
    failure_count: u32,
    last_failure: Option<Instant>,
}

impl NetworkErrorHandler {
    pub async fn handle_network_error(
        &mut self,
        operation: NetworkOperation,
        error: NetworkError,
    ) -> Result<NetworkResponse, NetworkError> {
        let endpoint = error.endpoint.clone();
        
        // Check circuit breaker
        if let Some(breaker) = self.circuit_breakers.get_mut(&endpoint) {
            if !breaker.is_open() {
                return self.fallback_handlers[&operation].execute();
            }
        }
        
        // Get retry policy
        let policy = &self.retry_policies[&operation];
        let mut attempt = 0;
        let mut delay = policy.base_delay;
        
        loop {
            attempt += 1;
            
            match self.attempt_operation(&operation).await {
                Ok(response) => {
                    self.record_success(&endpoint);
                    return Ok(response);
                }
                Err(e) if attempt < policy.max_attempts => {
                    self.record_failure(&endpoint);
                    
                    // Calculate delay with exponential backoff
                    if policy.jitter {
                        delay = self.add_jitter(delay);
                    }
                    
                    tokio::time::sleep(delay).await;
                    
                    delay = (delay.as_secs_f32() * policy.backoff_factor)
                        .min(policy.max_delay.as_secs_f32());
                    delay = Duration::from_secs_f32(delay);
                }
                Err(e) => {
                    self.record_failure(&endpoint);
                    
                    // Try fallback
                    if let Some(fallback) = self.fallback_handlers.get(&operation) {
                        return fallback.execute();
                    }
                    
                    return Err(e);
                }
            }
        }
    }
}
```

### Resource Error Handling

```rust
pub struct ResourceErrorHandler {
    fallback_resources: HashMap<String, ResourceFallback>,
    resource_cache: ResourceCache,
    placeholder_generator: PlaceholderGenerator,
}

pub enum ResourceFallback {
    StaticFallback(Vec<u8>),
    DynamicFallback(Box<dyn Fn() -> Vec<u8>>),
    PlaceholderGeneration(PlaceholderType),
    GracefulDegradation,
}

impl ResourceErrorHandler {
    pub fn handle_resource_error(
        &mut self,
        resource_id: &str,
        error: ResourceError,
    ) -> Result<Resource, ResourceError> {
        // Try cache first
        if let Some(cached) = self.resource_cache.get(resource_id) {
            return Ok(cached);
        }
        
        // Try fallback
        match self.fallback_resources.get(resource_id) {
            Some(ResourceFallback::StaticFallback(data)) => {
                Ok(Resource::from_bytes(data))
            }
            Some(ResourceFallback::DynamicFallback(generator)) => {
                let data = generator();
                Ok(Resource::from_bytes(&data))
            }
            Some(ResourceFallback::PlaceholderGeneration(placeholder_type)) => {
                let placeholder = self.placeholder_generator.generate(
                    placeholder_type,
                    resource_id
                );
                Ok(placeholder)
            }
            Some(ResourceFallback::GracefulDegradation) => {
                // Return a minimal valid resource
                Ok(Resource::minimal())
            }
            None => Err(error),
        }
    }
}

pub struct PlaceholderGenerator {
    texture_generator: TextureGenerator,
    audio_generator: AudioGenerator,
    model_generator: ModelGenerator,
}

impl PlaceholderGenerator {
    fn generate(&self, placeholder_type: &PlaceholderType, id: &str) -> Resource {
        match placeholder_type {
            PlaceholderType::Texture => {
                // Generate a simple colored texture
                let color = self.id_to_color(id);
                self.texture_generator.solid_color(64, 64, color)
            }
            PlaceholderType::Audio => {
                // Generate silence
                self.audio_generator.silence(Duration::from_secs(1))
            }
            PlaceholderType::Model => {
                // Generate a cube
                self.model_generator.cube(1.0)
            }
        }
    }
}
```

### Memory Error Handling

```rust
pub struct MemoryErrorHandler {
    memory_monitor: MemoryMonitor,
    pressure_handlers: Vec<MemoryPressureHandler>,
    oom_killer: OOMKiller,
}

pub struct MemoryMonitor {
    total_memory: usize,
    warning_threshold: f32,  // e.g., 0.8 = 80%
    critical_threshold: f32, // e.g., 0.95 = 95%
    check_interval: Duration,
}

pub struct MemoryPressureHandler {
    pressure_level: MemoryPressure,
    handler: Box<dyn Fn() -> usize>, // Returns bytes freed
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryPressure {
    Low,
    Medium,
    High,
    Critical,
}

impl MemoryErrorHandler {
    pub fn monitor_memory(&mut self) {
        let current_usage = self.memory_monitor.get_current_usage();
        let usage_ratio = current_usage as f32 / self.memory_monitor.total_memory as f32;
        
        let pressure = if usage_ratio >= self.memory_monitor.critical_threshold {
            MemoryPressure::Critical
        } else if usage_ratio >= self.memory_monitor.warning_threshold {
            MemoryPressure::High
        } else if usage_ratio >= 0.6 {
            MemoryPressure::Medium
        } else {
            MemoryPressure::Low
        };
        
        if pressure >= MemoryPressure::Medium {
            self.handle_memory_pressure(pressure);
        }
    }
    
    fn handle_memory_pressure(&mut self, pressure: MemoryPressure) {
        let mut freed = 0;
        
        for handler in &self.pressure_handlers {
            if handler.pressure_level <= pressure {
                freed += (handler.handler)();
            }
        }
        
        if pressure == MemoryPressure::Critical && freed < 1024 * 1024 * 10 {
            // Still critical after handlers, use OOM killer
            self.oom_killer.free_memory();
        }
    }
}

pub struct OOMKiller {
    killable_systems: Vec<KillableSystem>,
}

pub struct KillableSystem {
    name: String,
    priority: u32, // Lower = kill first
    estimated_memory: usize,
    kill_fn: Box<dyn Fn()>,
}

impl OOMKiller {
    fn free_memory(&mut self) {
        self.killable_systems.sort_by_key(|s| s.priority);
        
        for system in &self.killable_systems {
            eprintln!("OOM: Killing {} to free ~{}MB", 
                system.name, 
                system.estimated_memory / 1024 / 1024
            );
            (system.kill_fn)();
            
            // Check if we've freed enough
            if self.check_memory_available() {
                break;
            }
        }
    }
}
```

### Error Reporting & Analytics

```rust
pub struct ErrorReporter {
    report_queue: Arc<Mutex<VecDeque<ErrorReport>>>,
    uploader: Option<ErrorUploader>,
    local_storage: ErrorStorage,
    anonymizer: DataAnonymizer,
}

pub struct ErrorReport {
    error_id: Uuid,
    timestamp: DateTime<Utc>,
    error_type: String,
    severity: ErrorSeverity,
    context: ErrorContext,
    stack_trace: Option<String>,
    system_info: SystemInfo,
    user_id: Option<String>, // Anonymized
}

impl ErrorReporter {
    pub fn report_error(&mut self, error: &GameError, context: &ErrorContext) {
        let report = ErrorReport {
            error_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            error_type: format!("{:?}", error),
            severity: error.severity(),
            context: self.anonymizer.anonymize_context(context),
            stack_trace: self.capture_stack_trace(),
            system_info: self.get_system_info(),
            user_id: self.get_anonymous_user_id(),
        };
        
        // Store locally
        self.local_storage.store(&report);
        
        // Queue for upload if enabled
        if self.uploader.is_some() && self.should_report(&error) {
            self.report_queue.lock().unwrap().push_back(report);
        }
    }
    
    fn should_report(&self, error: &GameError) -> bool {
        match error.severity() {
            ErrorSeverity::Critical | ErrorSeverity::Severe => true,
            ErrorSeverity::Major => self.user_opted_in_major_errors(),
            _ => false,
        }
    }
}
```

### User-Facing Error UI

```rust
pub struct ErrorUI {
    error_dialog: ErrorDialog,
    notification_system: NotificationSystem,
    recovery_wizard: RecoveryWizard,
}

pub struct ErrorDialog {
    title: String,
    message: String,
    details: Option<String>,
    actions: Vec<ErrorAction>,
    style: DialogStyle,
}

pub enum ErrorAction {
    Retry,
    Ignore,
    RestoreBackup,
    ReportBug,
    Quit,
    RestartGame,
    OpenSettings,
}

impl ErrorUI {
    pub fn show_error(&mut self, error: &GameError, can_recover: bool) {
        match error.severity() {
            ErrorSeverity::Critical => {
                self.show_critical_error_dialog(error);
            }
            ErrorSeverity::Severe => {
                self.show_severe_error_dialog(error, can_recover);
            }
            ErrorSeverity::Major => {
                self.show_major_error_notification(error);
            }
            ErrorSeverity::Minor => {
                self.show_minor_error_toast(error);
            }
            ErrorSeverity::Warning => {
                // Log only, don't show UI
            }
        }
    }
    
    fn show_critical_error_dialog(&mut self, error: &GameError) {
        let dialog = ErrorDialog {
            title: "Critical Error".to_string(),
            message: "The game has encountered a critical error and must close.".to_string(),
            details: Some(format!("{:#?}", error)),
            actions: vec![
                ErrorAction::ReportBug,
                ErrorAction::Quit,
            ],
            style: DialogStyle::Critical,
        };
        
        self.error_dialog.show(dialog);
    }
}
```

## Error Prevention Strategies

### Defensive Programming

```rust
// Input validation
pub fn validate_input<T: Validate>(input: T) -> Result<T, ValidationError> {
    input.validate()?;
    Ok(input)
}

// Bounds checking
pub fn safe_array_access<T>(array: &[T], index: usize) -> Option<&T> {
    array.get(index)
}

// Resource limits
pub struct ResourceLimiter {
    max_creatures: usize,
    max_memory: usize,
    max_save_size: usize,
}

// Graceful degradation
pub fn with_fallback<T, F, G>(primary: F, fallback: G) -> T
where
    F: FnOnce() -> Result<T, GameError>,
    G: FnOnce() -> T,
{
    primary().unwrap_or_else(|_| fallback())
}
```

## Testing Error Scenarios

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_save_corruption_recovery() {
        let mut recovery = SaveErrorRecovery::new();
        let corrupted_save = create_corrupted_save();
        
        let result = recovery.handle_save_corruption(
            &corrupted_save,
            SaveError::ChecksumMismatch
        );
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_memory_pressure_handling() {
        let mut handler = MemoryErrorHandler::new();
        
        // Simulate high memory usage
        handler.memory_monitor.simulate_usage(0.9);
        handler.monitor_memory();
        
        // Verify handlers were called
        assert!(handler.get_freed_memory() > 0);
    }
    
    #[test]
    fn test_network_retry_with_backoff() {
        let mut handler = NetworkErrorHandler::new();
        
        // Simulate failures then success
        let result = handler.handle_network_error_mock(
            NetworkOperation::FetchData,
            vec![Err(()), Err(()), Ok(NetworkResponse::Success)]
        );
        
        assert!(result.is_ok());
        assert_eq!(handler.get_retry_count(), 2);
    }
}
```

## Integration Points

- **Logging System**: All errors are logged with full context
- **Save System**: Automatic backup creation on errors
- **UI System**: User-friendly error messages and recovery options
- **Performance System**: Memory pressure monitoring
- **Network System**: Retry and fallback mechanisms

## Performance Considerations

- Error handling adds < 1% overhead in normal operation
- Stack traces are captured asynchronously
- Error reports are batched for upload
- Memory pressure checks run on separate thread
- Recovery operations use progressive strategies