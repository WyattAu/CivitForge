- Status: accepted; steps 1-3 shipped (complete) (otelkit subscriber `7e20ce3`;
  official SDK; touching the request hot path.
1. otelkit; DONE step 1: `otelkit::init(TelemetryConfig)` replaced the
   middleware records real OTel instruments; the previous
2. flag-kit; DONE step 1: kit `bucket()` rollout + `FlagName`
   (`d191e50`). The adapter is deliberately a read model; `set`/`delete`