#!/usr/bin/env bash

# Set up a Linux machine for reproducible benchmarking.
#
# https://github.com/MoatLab/Pond/blob/master/cxl-global.sh
# https://easyperf.net/blog/2019/08/02/Perf-measurement-environment-on-Linux

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

# Kernel settings: https://docs.kernel.org/admin-guide/sysctl/kernel.html
readonly KERNEL="/proc/sys/kernel"

# Disable NMI watchdog
# https://docs.kernel.org/admin-guide/lockup-watchdogs.html
echo 0 | sudo tee "$KERNEL/nmi_watchdog"

# Disable NUMA balancing
# https://docs.kernel.org/admin-guide/sysctl/kernel.html#numa-balancing
echo 0 | sudo tee "$KERNEL/numa_balancing"

# Disable kernel pointer restrictions
# https://docs.kernel.org/admin-guide/sysctl/kernel.html#kptr-restrict
echo 0 | sudo tee "$KERNEL/kptr_restrict"

# Disable perf event restrictions
# https://docs.kernel.org/admin-guide/sysctl/kernel.html#perf-event-paranoid
echo "-1" | sudo tee "$KERNEL/perf_event_paranoid"

# Disable kernel samepage merging
# https://docs.kernel.org/admin-guide/mm/ksm.html
echo 0 | sudo tee "/sys/kernel/mm/ksm/run"

# CPU settings: https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-devices-system-cpu
readonly CPU="/sys/devices/system/cpu"

# Disable turbo boost
# https://en.wikipedia.org/wiki/Intel_Turbo_Boost
# https://www.kernel.org/doc/html/v5.0/admin-guide/pm/intel_pstate.html
if [[ -d "$CPU/intel_pstate" ]]; then
    echo 1 | sudo tee "$CPU/intel_pstate/no_turbo" || true
else
    echo 0 | sudo tee "$CPU/cpufreq/boost" || true
fi

# Set performance mode
# https://docs.kernel.org/admin-guide/pm/cpufreq.html
echo "performance" | sudo tee "$CPU/cpu*{0..9}/cpufreq/scaling_governor" || true

# Disable SMT
# https://en.wikipedia.org/wiki/Simultaneous_multithreading
if [[ -d "$CPU/smt" ]]; then
    echo "off" | sudo tee "$CPU/smt/control"
fi
