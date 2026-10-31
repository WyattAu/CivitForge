- `CreateRepository`; includes repository metadata (name, description, visibility, default branch)
- `ForkRepository`; references source and target repository URIs
- `ArchiveRepository`; new activity type for repository archival
- `TransferRepository`; new activity type for ownership transfer
- `CreateIssue`; includes title, body, labels, assignees, milestone references
- `CreatePullRequest`; includes source/target branches, diff metadata, merge strategy
- `ReviewPullRequest`; includes review state (Approved, ChangesRequested, Comment), inline comments
- `Comment`; generic commenting on any entity (issue, PR, commit, review)
- `StarRepository`; repository starrer graph
- `FollowUser`; user follow graph
- `Like`; reaction to any entity
- `Accept` / `Reject`; follow request acceptance
- `Undo`; reversal of any prior activity
1. If `remote_vc.happened_before(local_vc)`; delta is stale, skip
2. If `local_vc.happened_before(remote_vc)`; delta is causally newer, apply
3. If `local_vc.is_concurrent(remote_vc)`; concurrent writes, apply CRDT conflict resolution
| Stars | OR-Set (implicit) | - | Automatic convergence; no manual intervention needed |
| Issues | OR-Set (implicit) | - | Concurrent creates produce duplicate issues; deduplication via content hash |
| Pull Requests | OR-Set (implicit) | - | Same as issues |
| PR Reviews | LastWriteWins (per reviewer) | - | Each reviewer's review tracked independently |
| Comments | Sequence CRDT | - | No conflict; concurrent comments merged in causal order |
| Pipelines | Add-Wins Set | - | Additive only; deletion requires coordination |
| Labels | OR-Set (implicit) | - | Automatic convergence |
- `FEDERATION_REGION_ID`; unique identifier for this region
- `FEDERATION_REGIONS`; comma-separated list of peer region endpoints
- `FEDERATION_DISCOVERY_METHOD`; `config` or `dns`
| CRDT | Conflict-free Replicated Data Type; data structure that converges automatically under concurrent updates |
| LWW | Last-Writer-Wins; conflict resolution strategy where the most recent write by timestamp wins |
| OR-Set | Observed-Remove Set; CRDT supporting add/remove with add-wins semantics |
| RGA | Replicated Growable Array; sequence CRDT for ordered collections |