-- Down migration for 011: OCI Container Registry tables
-- Drops all OCI tables and indexes in reverse dependency order.

DROP TABLE IF EXISTS oci_policies;
DROP TABLE IF EXISTS oci_vuln_scans;
DROP TABLE IF EXISTS oci_image_signatures;
DROP TABLE IF EXISTS oci_manifest_layers;
DROP TABLE IF EXISTS oci_tags;
DROP TABLE IF EXISTS oci_manifests;
DROP TABLE IF EXISTS oci_blobs;
DROP TABLE IF EXISTS oci_repositories;
