//! Deterministic resource-pressure decisions for OriginWeave agent workloads.
//!
//! The governor protects human-visible rendering before agent throughput. It
//! does not allocate memory or schedule threads itself; platform adapters apply
//! the returned mitigation plan to Chromium, model, and observation processes.
//! Adapters include the CPU workers currently reserved by the task in every
//! snapshot so worker-pool saturation closes new admission deterministically.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::fmt;

const MEBIBYTE_BYTES: u64 = 1_048_576;

const fn bytes_to_mebibytes_ceil(bytes: u64) -> u64 {
    let whole_mebibytes = bytes / MEBIBYTE_BYTES;
    if bytes.is_multiple_of(MEBIBYTE_BYTES) {
        whole_mebibytes
    } else {
        whole_mebibytes + 1
    }
}

/// A validation error in a resource budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetError {
    /// At least one required limit was zero.
    ZeroLimit,
    /// A soft memory limit exceeded its corresponding hard limit.
    SoftExceedsHard,
}

impl fmt::Display for BudgetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroLimit => formatter.write_str("resource budget limits must be nonzero"),
            Self::SoftExceedsHard => {
                formatter.write_str("resource budget soft limits must not exceed hard limits")
            }
        }
    }
}

impl std::error::Error for BudgetError {}

/// Validated resource limits for one agent task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceBudget {
    soft_ram_mebibytes: u64,
    hard_ram_mebibytes: u64,
    soft_vram_mebibytes: u64,
    hard_vram_mebibytes: u64,
    cpu_threads: u16,
    frame_budget_milliseconds: u16,
}

impl ResourceBudget {
    /// Validate and create a task-level resource budget.
    pub const fn new(
        soft_ram_mebibytes: u64,
        hard_ram_mebibytes: u64,
        soft_vram_mebibytes: u64,
        hard_vram_mebibytes: u64,
        cpu_threads: u16,
        frame_budget_milliseconds: u16,
    ) -> Result<Self, BudgetError> {
        if soft_ram_mebibytes == 0
            || hard_ram_mebibytes == 0
            || soft_vram_mebibytes == 0
            || hard_vram_mebibytes == 0
            || cpu_threads == 0
            || frame_budget_milliseconds == 0
        {
            return Err(BudgetError::ZeroLimit);
        }
        if soft_ram_mebibytes > hard_ram_mebibytes || soft_vram_mebibytes > hard_vram_mebibytes {
            return Err(BudgetError::SoftExceedsHard);
        }
        Ok(Self {
            soft_ram_mebibytes,
            hard_ram_mebibytes,
            soft_vram_mebibytes,
            hard_vram_mebibytes,
            cpu_threads,
            frame_budget_milliseconds,
        })
    }

    /// Return the RAM pressure threshold in mebibytes.
    #[must_use]
    pub const fn soft_ram_mebibytes(self) -> u64 {
        self.soft_ram_mebibytes
    }

    /// Return the RAM stop threshold in mebibytes.
    #[must_use]
    pub const fn hard_ram_mebibytes(self) -> u64 {
        self.hard_ram_mebibytes
    }

    /// Return the VRAM pressure threshold in mebibytes.
    #[must_use]
    pub const fn soft_vram_mebibytes(self) -> u64 {
        self.soft_vram_mebibytes
    }

    /// Return the VRAM stop threshold in mebibytes.
    #[must_use]
    pub const fn hard_vram_mebibytes(self) -> u64 {
        self.hard_vram_mebibytes
    }

    /// Return the maximum fixed worker count allocated to the task.
    #[must_use]
    pub const fn cpu_threads(self) -> u16 {
        self.cpu_threads
    }

    /// Return the maximum tolerated compositor frame time in milliseconds.
    #[must_use]
    pub const fn frame_budget_milliseconds(self) -> u16 {
        self.frame_budget_milliseconds
    }
}

/// A point-in-time measurement of one agent task's resource use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceSnapshot {
    ram_mebibytes: u64,
    vram_mebibytes: u64,
    agent_batch_size: u32,
    local_model_loaded: bool,
    frame_time_milliseconds: u16,
    cpu_threads_in_use: u16,
}

impl ResourceSnapshot {
    /// Create one resource observation collected by a platform adapter.
    ///
    /// `cpu_threads_in_use` is the number of CPU workers already reserved by
    /// this task at the decision point. It is observation data, not a request
    /// to create or resize a worker pool.
    #[must_use]
    pub const fn new(
        ram_mebibytes: u64,
        vram_mebibytes: u64,
        agent_batch_size: u32,
        local_model_loaded: bool,
        frame_time_milliseconds: u16,
        cpu_threads_in_use: u16,
    ) -> Self {
        Self {
            ram_mebibytes,
            vram_mebibytes,
            agent_batch_size,
            local_model_loaded,
            frame_time_milliseconds,
            cpu_threads_in_use,
        }
    }

    /// Build a governor snapshot using adapter-supplied browser/task RSS telemetry.
    ///
    /// Supplied browser RSS bytes are rounded up to mebibytes so a partial
    /// mebibyte is never understated at a configured pressure boundary. VRAM,
    /// batch size, local-model state, frame time, and CPU-worker use remain
    /// explicit adapter observations; this conversion does not infer any of
    /// them from browser telemetry.
    #[must_use]
    pub const fn from_browser_task_telemetry(
        telemetry: BrowserTaskTelemetry,
        vram_mebibytes: u64,
        agent_batch_size: u32,
        local_model_loaded: bool,
        frame_time_milliseconds: u16,
        cpu_threads_in_use: u16,
    ) -> Self {
        Self::new(
            bytes_to_mebibytes_ceil(telemetry.browser_rss_bytes()),
            vram_mebibytes,
            agent_batch_size,
            local_model_loaded,
            frame_time_milliseconds,
            cpu_threads_in_use,
        )
    }
}

/// Independent mitigations that a platform adapter applies to one workload.
///
/// A plan can carry several actions at once. This prevents simultaneous RAM,
/// VRAM, and frame pressure from being collapsed into a single enum variant.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResourceMitigationPlan {
    spill_observation_cache: bool,
    next_batch_size: Option<u32>,
    offload_inference_to_cpu: bool,
    pause_current_agent: bool,
    reject_new_agent_work: bool,
}

impl ResourceMitigationPlan {
    const fn new(
        spill_observation_cache: bool,
        next_batch_size: Option<u32>,
        offload_inference_to_cpu: bool,
        pause_current_agent: bool,
        reject_new_agent_work: bool,
    ) -> Self {
        Self {
            spill_observation_cache,
            next_batch_size,
            offload_inference_to_cpu,
            pause_current_agent,
            reject_new_agent_work,
        }
    }

    /// Return whether old semantic observations must leave process RAM.
    #[must_use]
    pub const fn spill_observation_cache(self) -> bool {
        self.spill_observation_cache
    }

    /// Return the reduced batch size for the next agent step, when required.
    #[must_use]
    pub const fn next_batch_size(self) -> Option<u32> {
        self.next_batch_size
    }

    /// Return whether local inference must release GPU memory and use CPU.
    #[must_use]
    pub const fn offload_inference_to_cpu(self) -> bool {
        self.offload_inference_to_cpu
    }

    /// Return whether the currently running agent must stop making progress.
    #[must_use]
    pub const fn pause_current_agent(self) -> bool {
        self.pause_current_agent
    }

    /// Return whether admission of additional agent work must be rejected.
    #[must_use]
    pub const fn reject_new_agent_work(self) -> bool {
        self.reject_new_agent_work
    }

    /// Return whether the workload may continue without any mitigation.
    #[must_use]
    pub const fn is_noop(self) -> bool {
        !self.spill_observation_cache
            && self.next_batch_size.is_none()
            && !self.offload_inference_to_cpu
            && !self.pause_current_agent
            && !self.reject_new_agent_work
    }
}

/// A pure resource-governance policy bound to one validated budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceGovernor {
    budget: ResourceBudget,
}

impl ResourceGovernor {
    /// Create a governor for one task budget.
    #[must_use]
    pub const fn new(budget: ResourceBudget) -> Self {
        Self { budget }
    }

    /// Build the complete mitigation plan for one resource snapshot.
    ///
    /// Hard memory pressure always pauses the active agent and rejects new
    /// admission. Hard VRAM pressure also unloads a resident local model so the
    /// plan reduces the consumer that crossed the configured limit. CPU worker
    /// saturation closes only new admission; it does not pause the active task
    /// unless another pressure rule independently requires that action.
    #[must_use]
    pub const fn decide(self, snapshot: ResourceSnapshot) -> ResourceMitigationPlan {
        let hard_ram = snapshot.ram_mebibytes >= self.budget.hard_ram_mebibytes;
        let hard_vram = snapshot.vram_mebibytes >= self.budget.hard_vram_mebibytes;
        let frame_pressure =
            snapshot.frame_time_milliseconds >= self.budget.frame_budget_milliseconds;
        let soft_ram = snapshot.ram_mebibytes >= self.budget.soft_ram_mebibytes;
        let soft_vram = snapshot.vram_mebibytes >= self.budget.soft_vram_mebibytes;
        let cpu_saturated = snapshot.cpu_threads_in_use >= self.budget.cpu_threads;

        let spill_observation_cache = soft_ram;
        let reject_new_agent_work = hard_ram || hard_vram || cpu_saturated;
        let mut pause_current_agent = hard_ram || hard_vram;
        let mut offload_inference_to_cpu = hard_vram && snapshot.local_model_loaded;
        let mut next_batch_size = None;

        if frame_pressure {
            if snapshot.local_model_loaded {
                offload_inference_to_cpu = true;
            } else {
                pause_current_agent = true;
            }
        }

        if soft_vram && !hard_vram {
            if snapshot.agent_batch_size > 1 {
                next_batch_size = Some(snapshot.agent_batch_size / 2);
            } else if snapshot.local_model_loaded {
                offload_inference_to_cpu = true;
            } else {
                pause_current_agent = true;
            }
        }

        ResourceMitigationPlan::new(
            spill_observation_cache,
            next_batch_size,
            offload_inference_to_cpu,
            pause_current_agent,
            reject_new_agent_work,
        )
    }
}

/// Validated browser-task resource measurements supplied by a platform adapter.
///
/// The producer is responsible for obtaining these values from its own trusted
/// measurement boundary. This value type validates relationships and bounds only;
/// it does not sample the operating system or Chromium or prove measurement
/// provenance. It stores no page content, credentials, GPU state, model identity,
/// or persistence metadata and never infers local-AI usage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrowserTaskTelemetry {
    browser_rss_bytes: u64,
    observation_bytes: u64,
    action_latency_milliseconds: u64,
    task_duration_milliseconds: u64,
}

impl BrowserTaskTelemetry {
    /// Validate one bounded platform-supplied browser-task telemetry record.
    ///
    /// Browser RSS and total task duration must be nonzero. An empty semantic
    /// observation is valid. Action latency may be zero, but cannot exceed the
    /// total task duration reported for the same execution interval.
    pub const fn new(
        browser_rss_bytes: u64,
        observation_bytes: u64,
        action_latency_milliseconds: u64,
        task_duration_milliseconds: u64,
    ) -> Result<Self, BrowserTaskTelemetryError> {
        if browser_rss_bytes == 0 {
            return Err(BrowserTaskTelemetryError::ZeroBrowserRss);
        }
        if task_duration_milliseconds == 0 {
            return Err(BrowserTaskTelemetryError::ZeroTaskDuration);
        }
        if action_latency_milliseconds > task_duration_milliseconds {
            return Err(
                BrowserTaskTelemetryError::ActionLatencyExceedsTaskDuration {
                    action_latency_milliseconds,
                    task_duration_milliseconds,
                },
            );
        }
        Ok(Self {
            browser_rss_bytes,
            observation_bytes,
            action_latency_milliseconds,
            task_duration_milliseconds,
        })
    }

    /// Return the supplied resident-set size for the browser/task process set.
    #[must_use]
    pub const fn browser_rss_bytes(self) -> u64 {
        self.browser_rss_bytes
    }

    /// Return the supplied number of bytes in the bounded semantic observation.
    #[must_use]
    pub const fn observation_bytes(self) -> u64 {
        self.observation_bytes
    }

    /// Return the supplied latency of the governed browser action.
    #[must_use]
    pub const fn action_latency_milliseconds(self) -> u64 {
        self.action_latency_milliseconds
    }

    /// Return the supplied duration of the complete browser-task interval.
    #[must_use]
    pub const fn task_duration_milliseconds(self) -> u64 {
        self.task_duration_milliseconds
    }
}

/// A reason that browser-task telemetry cannot enter the trusted resource record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserTaskTelemetryError {
    /// Browser/task resident-set size was zero and therefore unusable.
    ZeroBrowserRss,
    /// Total task duration was zero and therefore not a usable interval.
    ZeroTaskDuration,
    /// One action was reported as taking longer than the enclosing task interval.
    ActionLatencyExceedsTaskDuration {
        /// Reported governed-action latency in milliseconds.
        action_latency_milliseconds: u64,
        /// Reported complete task duration in milliseconds.
        task_duration_milliseconds: u64,
    },
}

/// A reason that a Linux process resident-set-size sample could not be obtained safely.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserRssSampleError {
    /// Process identifiers are one-based and zero was supplied.
    InvalidProcessId,
    /// The process status file could not be read at the sampling boundary.
    ProcessStatusUnavailable,
    /// The Linux process status did not contain a resident-set-size field.
    MissingVmRss,
    /// More than one resident-set-size field was present in the status record.
    DuplicateVmRss,
    /// The resident-set-size field was syntactically malformed.
    InvalidVmRss,
    /// The resident-set-size field did not use the Linux kernel `kB` unit.
    UnsupportedVmRssUnit,
    /// Converting the kernel kibibyte count to bytes would overflow `u64`.
    VmRssOverflow,
    /// The current operating system does not expose Linux `/proc` process status.
    UnsupportedPlatform,
}

/// Parse Linux `/proc/<pid>/status` and return the exact `VmRSS` value in bytes.
///
/// Linux reports `VmRSS` in `kB`, where the kernel ABI uses 1024-byte units.
/// The parser accepts exactly one `VmRSS:` record with one integer and the
/// literal `kB` unit, rejects ambiguous duplicates or trailing fields, and uses
/// checked multiplication so an untrusted status payload cannot wrap the byte count.
pub fn parse_linux_proc_status_rss_bytes(status: &str) -> Result<u64, BrowserRssSampleError> {
    let mut resident_kibibytes = None;

    for line in status.lines() {
        let Some(value_text) = line.strip_prefix("VmRSS:") else {
            continue;
        };
        if resident_kibibytes.is_some() {
            return Err(BrowserRssSampleError::DuplicateVmRss);
        }

        let mut fields = value_text.split_whitespace();
        let Some(raw_value) = fields.next() else {
            return Err(BrowserRssSampleError::InvalidVmRss);
        };
        let Some(unit) = fields.next() else {
            return Err(BrowserRssSampleError::InvalidVmRss);
        };
        if fields.next().is_some() {
            return Err(BrowserRssSampleError::InvalidVmRss);
        }
        if unit != "kB" {
            return Err(BrowserRssSampleError::UnsupportedVmRssUnit);
        }
        if !raw_value.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(BrowserRssSampleError::InvalidVmRss);
        }
        let parsed = raw_value
            .parse::<u64>()
            .map_err(|_error| BrowserRssSampleError::InvalidVmRss)?;
        resident_kibibytes = Some(parsed);
    }

    let resident_kibibytes = resident_kibibytes.ok_or(BrowserRssSampleError::MissingVmRss)?;
    resident_kibibytes
        .checked_mul(1_024)
        .ok_or(BrowserRssSampleError::VmRssOverflow)
}

/// Sample one operating-system process resident set in bytes from Linux `/proc`.
///
/// The caller supplies the exact browser process identifier it owns. This
/// function performs no process discovery and follows no browser-child tree;
/// it only reads `/proc/<pid>/status` for that identifier. Non-Linux platforms
/// fail closed with [`BrowserRssSampleError::UnsupportedPlatform`].
pub fn sample_linux_process_rss_bytes(process_id: u32) -> Result<u64, BrowserRssSampleError> {
    if process_id == 0 {
        return Err(BrowserRssSampleError::InvalidProcessId);
    }

    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string(format!("/proc/{process_id}/status"))
            .map_err(|_error| BrowserRssSampleError::ProcessStatusUnavailable)?;
        parse_linux_proc_status_rss_bytes(&status)
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = process_id;
        Err(BrowserRssSampleError::UnsupportedPlatform)
    }
}
