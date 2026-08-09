//! Dispatch (Anubis) — SOLID discipline gate.
//!
//! Pure contract gate. ZERO I/O. No CBM, no grep, no file reads.
//! Context comes from scan_context (which always runs before dispatch).
//! Dispatch only wraps the task with SOLID constraints — <1ms.

use serde::Serialize;

const SOLID_PRINCIPLES: &str = r#"## SOLID Principles (MANDATORY)

1. **Single Responsibility** — each module/class/function has ONE reason to change.
   Split god classes into focused units. Each file should do exactly one thing well.
2. **Open-Closed** — extend behavior via composition/plugins, never modify existing code.
   Add new files for new features; don't cram into existing ones.
3. **Liskov Substitution** — subtypes must be substitutable for their base types.
   If it extends something, it must honor the full contract.
4. **Interface Segregation** — no client should depend on methods it doesn't use.
   Keep interfaces small and focused. Split fat interfaces.
5. **Dependency Inversion** — depend on abstractions (traits/interfaces), not concretions.
   Inject dependencies; don't hardcode implementations.

## Code Quality (MANDATORY)

- **Zero duplication** — extract shared logic, never copy-paste
- **No magic numbers** — all constants named and documented
- **Meaningful names** — variables, functions, files tell their purpose
- **YAGNI** — only build what's needed now
- **Self-documenting code** — comments explain WHY, not WHAT
- **Error handling** — every fallible operation has explicit error handling
- **Tests alongside** — new code requires tests (TDD: RED → GREEN → REFACTOR)

## Observability (MANDATORY)

- **Every HTTP request MUST produce exactly ONE structured log row** — JSON, single line:
  `{"level":"info","msg":"request completed","method":"POST","path":"/api/auth/login","status":200,"duration_ms":42}`
  Never split one request across multiple lines (breaks log aggregators).
- **Use the framework's BUILT-IN logging first — no extra library unless needed:**
  - NestJS → built-in `Logger` from `@nestjs/common` (per-module: `private readonly logger = new Logger(AuthService.name)`). Zero dependencies.
  - Express/Fastify → create a logging middleware FIRST (before routes) so every request is captured — custom middleware or pino-http. No scattered `console.log`.
  - Go/Python/etc → stdlib logger or framework default first; structured libs (zap/structlog) only when built-in is insufficient.
- **Log levels:** INFO = business events/lifecycle, WARN = suspicious/retryable, ERROR = failures WITH stack + context (what failed, inputs, error).
- **Every catch block MUST log** — silent catches are forbidden. Log with context, never bare `console.log(error)`.
- **Never log:** passwords, tokens, secrets, personal data. Mask/redact if unavoidable.
- **One logger instance** — no ad-hoc loggers scattered across files; inject/import a single configured logger."#;

#[derive(Debug, Serialize)]
pub struct SolidityContract {
    pub task: String,
    pub constraints: String,
    pub gate: String,
}

/// Generate SOLID contract. ZERO I/O.
/// Context was already provided by scan_context — dispatch only enforces discipline.
pub fn generate_contract(task: &str) -> SolidityContract {
    SolidityContract {
        task: task.to_string(),
        constraints: SOLID_PRINCIPLES.to_string(),
        gate: "🎯 EXECUTE with SOLID. No shortcuts. No god classes. No copy-paste. \
               No magic numbers. Every module has ONE reason to change. \
               Extend via composition — never modify existing code. \
               Tests alongside every new file. READ the contract above before writing."
            .to_string(),
    }
}
