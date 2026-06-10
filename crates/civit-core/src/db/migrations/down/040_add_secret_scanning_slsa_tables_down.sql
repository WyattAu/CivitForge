DROP INDEX IF EXISTS idx_slsa_attestations_unique;
DROP INDEX IF EXISTS idx_slsa_attestations_run;
DROP INDEX IF EXISTS idx_slsa_attestations_repo;
DROP TABLE IF EXISTS slsa_attestations;

DROP INDEX IF EXISTS idx_secret_scan_results_scan_id;
DROP INDEX IF EXISTS idx_secret_scan_results_repo;
DROP TABLE IF EXISTS secret_scan_results;
