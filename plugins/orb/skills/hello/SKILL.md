---
name: hello
description: Smoke test skill — greets the user and confirms orbit plugin is installed
disable-model-invocation: false
---

# Hello from Orbit

You are running a smoke test for the orbit plugin.

Respond with:
1. "✅ Orbit plugin is installed and working"
2. The skill namespace you were invoked as (should be `/orb:hello`)
3. List any other `orb:` skills you can see in your skill listing

This confirms:
- The marketplace resolved correctly
- The plugin namespace is `orb:` (not `orbit:`)
- Skills are discoverable
