# Approve tip freeze

Do not embed a self-referential commit hash in-repo (hash changes when the file is committed).

Machine check at close:

```bash
gh pr view 470 --json headRefOid -q .headRefOid
jq -r .commit_id /tmp/grok-goal-40c764f701ea/implementer/liukongqiang5-approve-readback.json
# must be equal; reviewer liukongqiang5; state APPROVED
```
