//! Deterministic resource-pressure decisions for OriginWeave agent workloads.
//!
//! The governor protects human-visible rendering before agent throughput. It
//! does not allocate memory or schedule threads itself; platform adapters apply
//! the returned mitigation plan to Chromium, model, and observation processes.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// A validation error in a resource budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetError {
    /// At least one required limit was zero.
    ZeroLimit,
    /// A soft memory limit exceeded its corresponding hard limit.
    SoftExceedsHard,
}

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
}

impl ResourceSnapshot {
    /// Create one resource observation collected by a platform adapter.
    #[must_use]
    pub const fn new(
        ram_mebibytes: u64,
        vram_mebibytes: u64,
        agent_batch_size: u32,
        local_model_loaded: bool,
        frame_time_milliseconds: u16,
    ) -> Self {
        Self {
            ram_mebibytes,
            vram_mebibytes,
            agent_batch_size,
            local_model_loaded,
            frame_time_milliseconds,
        }
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
    /// plan reduces the consumer that crossed the configured limit.
    #[must_use]
    pub const fn decide(self, snapshot: ResourceSnapshot) -> ResourceMitigationPlan {
        let hard_ram = snapshot.ram_mebibytes >= self.budget.hard_ram_mebibytes;
        let hard_vram = snapshot.vram_mebibytes >= self.budget.hard_vram_mebibytes;
        let frame_pressure =
            snapshot.frame_time_milliseconds >= self.budget.frame_budget_milliseconds;
        let soft_ram = snapshot.ram_mebibytes >= self.budget.soft_ram_mebibytes;
        let soft_vram = snapshot.vram_mebibytes >= self.budget.soft_vram_mebibytes;

        let spill_observation_cache = soft_ram;
        let reject_new_agent_work = hard_ram || hard_vram;
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
