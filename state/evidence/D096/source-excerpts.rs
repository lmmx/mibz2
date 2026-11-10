pub fn new(threads: usize, slots: usize, k: Kernels) -> StreamDecoder {
        let threads = threads.max(1);
        let forced = FORCED.with(|c| c.get()).is_some();
        // (the cheap conditions first: the machine is read only at the target's budget)
        let dec = if !forced
            && threads == SERVICE_THREADS
            && k.avx2
            && k.clmul
            && service_target(threads, k, &Machine::here())
        {
            let p = SERVICE_PLAN;
            let slots = window(p.as_plan(), threads, slots);
            Self::service(p.teams, p.k, p.services, slots, k)
        } else {
            let plan = Plan::for_threads(threads);
            Self::defaulted(plan, window(plan, threads, slots), k)
        };
        record_built(&dec);
        dec
    }


pub fn service_target(threads: usize, k: Kernels, m: &Machine) -> bool {
    threads == SERVICE_THREADS
        && k.avx2
        && k.clmul
        && cfg!(all(target_os = "linux", target_arch = "x86_64"))
        && m.target.as_ref().is_some_and(|t| {
            (t.cpu.0.as_str(), t.cpu.1, t.cpu.2) == SERVICE_CPU
                && t.name == SERVICE_NAME
                && t.phys == TEN_CORES.0
                && t.logical == SERVICE_LOGICAL
                && t.l3 == TEN_CORES.1
        })
        && m.quota == Quota::Unlimited
}
