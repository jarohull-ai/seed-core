# SEED-Core Integration Guide
## Connecting SEED-Core to LangChain, LlamaIndex, and Custom Agents

This guide provides practical integration patterns for using **SEED-Core** as the Stage-2 Gatekeeper in popular autonomous agent frameworks.

---

## 1. Direct Python SDK Integration (Baseline)

The fastest way to protect any agent loop is evaluating directives immediately before tool execution:

```python
from seed_core import SeedDecider

# Initialize the Stage-2 Decider
decider = SeedDecider(profile="HOST_SOVEREIGN")

def safe_tool_executor(intent: str, action_type: str, target: str):
    # Evaluate with SEED-Core before execution (~15 µs latency)
    verdict = decider.evaluate(
        intent=intent,
        action_type=action_type,
        target=target
    )

    if verdict.is_approved:
        print(f"✅ Approved in {verdict.latency_us} µs. Running target...")
        # Execute tool safely
        return execute_tool_internally(target)
    elif verdict.is_vetoed:
        raise PermissionError(f"🚫 SEED-Core Veto: {verdict.reason}")
    elif verdict.requires_escalation:
        # Prompt human operator (EU AI Act Article 14 Human Oversight)
        user_confirmed = ask_operator_approval(verdict.reason)
        if user_confirmed:
            return execute_tool_internally(target)
        raise PermissionError("⚠️ User aborted escalated action.")
```

---

## 2. LangChain Tool Validation Pattern

Wrap standard LangChain tools with a deterministic SEED-Core guardrail:

```python
from langchain.tools import BaseTool
from seed_core import SeedDecider

decider = SeedDecider(profile="HOST_SOVEREIGN")

class SeedProtectedBashTool(BaseTool):
    name: str = "terminal_run"
    description: str = "Executes bash commands on the host environment under SEED-Core protection."

    def _run(self, command: str) -> str:
        # Evaluate command before dispatching to shell
        verdict = decider.evaluate(
            intent="Agent Terminal Execution",
            action_type="command_exec",
            target=command
        )

        if verdict.is_vetoed:
            return f"Error: Command rejected by SEED-Core security policy: {verdict.reason}"
        elif verdict.requires_escalation:
            return f"Action requires human administrator confirmation: {verdict.reason}"

        # Safe execution
        import subprocess
        result = subprocess.run(command, shell=True, capture_output=True, text=True)
        return result.stdout or result.stderr
```

---

## 3. LlamaIndex Guardrail Pattern

Hook SEED-Core into LlamaIndex query and tool workflows:

```python
from llama_index.core.tools import FunctionTool
from seed_core import SeedDecider

decider = SeedDecider(profile="HOST_SOVEREIGN")

def execute_workspace_command(cmd: str) -> str:
    verdict = decider.evaluate(
        intent="LlamaIndex Tool Call",
        action_type="command_exec",
        target=cmd
    )
    if not verdict.is_approved:
        return f"Policy Interception: {verdict.reason}"
    
    # Run command
    return "Executed successfully"

seed_tool = FunctionTool.from_defaults(
    fn=execute_workspace_command,
    name="workspace_runner",
    description="Deterministic command execution guarded by SEED-Core."
)
```

---

## 4. Node.js & TypeScript via Tokio IPC Daemon

For JavaScript, TypeScript, or Go agents (e.g. **Viper Chat**), connect to SEED-Core's background Unix socket daemon:

### 1. Launch SEED-Core Daemon:
```bash
seed-core serve --profile HOST_SOVEREIGN --socket /tmp/seed-core.sock
```

### 2. Client Integration (Node.js):
```typescript
import * as net from 'net';

function evaluateWithSeed(target: string): Promise<any> {
    return new Promise((resolve, reject) => {
        const client = net.createConnection({ path: '/tmp/seed-core.sock' }, () => {
            const request = {
                candidate: {
                    step_id: 1,
                    intent: 'Node.js Agent Action',
                    action_type: 'CommandExec',
                    action_args: { target },
                    planner_thought: 'Executing requested build'
                }
            };
            client.write(JSON.stringify(request) + '\n');
        });

        client.on('data', (data) => {
            const response = JSON.parse(data.toString());
            client.end();
            resolve(response.data);
        });

        client.on('error', (err) => reject(err));
    });
}
```
