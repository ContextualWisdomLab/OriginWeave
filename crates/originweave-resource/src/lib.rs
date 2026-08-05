//! Deterministic resource-pressure decisions for OriginWeave agent workloads.
//!
//! The governor protects human-visible rendering before agent throughput. It
//! does not allocate memory or schedule threads itself; platform adapters apply
//! the returned directive to Chromium, model, and observation processes.

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

/// A fail-closed action for the platform resource scheduler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceDirective {
    /// Continue the current browser and agent workload.
    Continue,
    /// Move old semantic observations from RAM to bounded persistent storage.
    SpillObservationCache,
    /// Re-run the next agent step with a smaller batch.
    ReduceAgentBatch {
        /// The bounded batch size to use for the next step.
        next_batch_size: u32,
    },
    /// Remove the local model from GPU memory and continue on CPU.
    OffloadInferenceToCpu,
    /// Suspend the current agent while preserving human browser interaction.
    PauseAgent,
    /// Reject new agent work until hard GPU pressure has cleared.
    RejectNewAgentWork,
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

    /// Select the highest-priority mitigation for one resource snapshot.
    #[must_use]
    pub const fn decide(self, snapshot: ResourceSnapshot) -> ResourceDirective {
        if snapshot.vram_mebibytes >= self.budget.hard_vram_mebibytes {
            return ResourceDirective::RejectNewAgentWork;
        }
        if snapshot.ram_mebibytes >= self.budget.hard_ram_mebibytes {
            return ResourceDirective::PauseAgent;
        }
        if snapshot.frame_time_milliseconds >= self.budget.frame_budget_milliseconds {
            return if snapshot.local_model_loaded {
                ResourceDirective::OffloadInferenceToCpu
            } else {
                ResourceDirective::PauseAgent
            };
        }
        if snapshot.ram_mebibytes >= self.budget.soft_ram_mebibytes {
            return ResourceDirective::SpillObservationCache;
        }
        if snapshot.vram_mebibytes >= self.budget.soft_vram_mebibytes {
            if snapshot.agent_batch_size > 1 {
                return ResourceDirective::ReduceAgentBatch {
                    next_batch_size: snapshot.agent_batch_size / 2,
                };
            }
            return if snapshot.local_model_loaded {
                ResourceDirective::OffloadInferenceToCpu
            } else {
                ResourceDirective::PauseAgent
            };
        }
        ResourceDirective::Continue
    }
}
