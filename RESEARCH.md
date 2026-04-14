# Beyond Git: Version Control for the AI Age

> A research document exploring why git's fundamental model breaks down when AI becomes
> the primary author of code, and what a successor system — built for human-AI collaboration
> from the ground up — could look like.

---

## Table of Contents

1. [The Problem: Why Git Breaks Down with AI](#the-problem-why-git-breaks-down-with-ai)
2. [The Vision: Intent-Based Version Control](#the-vision-intent-based-version-control)
3. [Human Oversight: Regaining Transparency and Ownership](#human-oversight-regaining-transparency-and-ownership)
4. [Collaboration: Humans, AIs, and Teams](#collaboration-humans-ais-and-teams)
5. [Technical Sketch: What This System Could Look Like](#technical-sketch-what-this-system-could-look-like)
6. [Conclusion: From Ledger to Living History](#conclusion-from-ledger-to-living-history)

---

## The Problem: Why Git Breaks Down with AI

Git was designed in 2005 for a world where humans typed code, line by line, over hours and days. That world is ending. As AI-assisted coding becomes the dominant mode of software development, Git's foundational assumptions are failing in ways that range from awkward to fundamentally broken. The tool still *works* — you can still commit, push, and merge — but the workflows built on top of it are increasingly mismatched with how code actually gets written.

### The commit granularity mismatch

Git's model rewards small, focused, atomic commits. A good commit changes one thing, for one reason, and its diff tells a coherent story. This ideal was already aspirational for most human developers, but it was at least *achievable*. With AI, it becomes almost absurd.

When you ask an AI to "refactor this module to use dependency injection," it may touch 30 files in eight seconds. The result is correct, coherent, and intentional — but it is a single act of creation. Do you commit it as one massive changeset? The diff will be enormous, effectively unreviewable in the traditional sense. Do you try to split it into smaller commits? That requires either manually staging hunks of an AI-generated change (tedious and error-prone) or asking the AI to artificially decompose its own work into sequential steps it never actually took. You are manufacturing a false history to satisfy a tool's assumptions.

Neither option is good. The single massive commit overwhelms reviewers. The artificial split creates a fiction — a sequence of "incremental" changes that never existed, authored by a process that doesn't work incrementally. Git's commit model assumes the *pace* of change matches the *granularity* of commits. AI violates that assumption completely.

### The diff problem

Git diffs are line-based. They show you that line 47 was deleted and a new line 47 was inserted. For human-authored changes, this is often sufficient: a developer changed a variable name, fixed an off-by-one error, added a null check. The diff *is* the explanation.

When an AI rewrites a function — or worse, rewrites an entire file to follow a different pattern — the diff becomes a wall of red and green. Every line is "changed." The structural relationship between the old code and the new code is invisible. A human reviewer staring at a 400-line diff of an AI-rewritten service class cannot tell which changes are load-bearing and which are stylistic. They cannot tell what the AI preserved intentionally versus what it dropped by accident. The diff shows *what* changed at the text level but communicates nothing about *why* or *how intent maps to the change*.

Consider a concrete example: you ask an AI to convert a React class component to a functional component with hooks. The diff will show the entire class deleted and an entirely new function added. Semantically, 80% of the logic is identical. But line-based diffing cannot surface that. A reviewer has to reconstruct the semantic mapping in their head, which largely defeats the purpose of a diff-based review process.

### The authorship and attribution problem

Git has an author field. It holds one name and one email address. This made perfect sense when one person wrote each commit. It makes no sense when a human writes a prompt, an AI generates 200 lines of code, and the human tweaks 3 of them.

Who is the author? The human, who provided the intent and validated the result? The AI, which wrote the vast majority of the actual characters? Git forces a binary choice. In practice, most teams default to listing the human, which means `git blame` now attributes AI-generated code to people who may barely understand it. The entire premise of blame — "this person can explain this line" — collapses. You page someone at 2 AM about a production bug in code an AI wrote three months ago, and they have no more context than you do.

Some teams have experimented with `Co-authored-by` trailers, but this is a convention, not a real attribution model. It does not capture the *degree* of contribution, the nature of the human's role (prompter? reviewer? editor?), or which specific lines came from whom. The metadata model is simply too impoverished for the reality of human-AI collaboration.

### The branching model friction

Feature branches, pull requests, rebasing, merge conflict resolution — the entire branching workflow assumes parallel human work happening over days or weeks. You create a branch because you need isolation while you think, experiment, and iterate. You open a PR because other humans need time to read and understand your changes. You resolve merge conflicts because two humans made overlapping decisions independently.

AI collapses the timeline. A feature that would have lived on a branch for a week gets generated in five minutes. The overhead of creating a branch, pushing it, opening a PR, waiting for CI, requesting review, and merging is now vastly disproportionate to the actual work. The ceremony that once provided necessary structure now introduces pure friction.

Merge conflicts become stranger too. When two AI-generated changes conflict, the resolution requires understanding two sets of AI reasoning, neither of which is recorded anywhere. A human resolving a conflict between two AI-generated refactors is essentially arbitrating between two opaque decision-making processes.

### The knowledge loss problem

This may be the most damaging failure. Git captures *what* changed (the diff) and *when* (the timestamp). It can optionally capture a brief *why* (the commit message). But it systematically loses the richest context surrounding any change.

In AI-assisted development, the most valuable artifact is often the *conversation*: the human's initial intent, the AI's interpretation, the back-and-forth refinement, the alternatives that were considered and rejected, the constraints that were articulated, the tradeoffs that were explicitly discussed. None of this survives into Git. The commit message might say "refactor auth module," but the conversation that produced it might have included: "I considered a token-based approach but rejected it because of our latency requirements; the session-based approach was chosen despite its scalability tradeoff because we don't expect more than 10,000 concurrent users in the next two years."

That reasoning — the *decision architecture* — is the most valuable thing for future maintainers. And it is completely discarded. Git stores the *output* of a decision process but none of the *process itself*. When code was written slowly by humans, much of that reasoning lived in the developer's head and could be recovered by asking them. When code is generated by AI in a conversation that gets closed and forgotten, the reasoning is gone permanently.

This is not a minor inconvenience. It is a structural loss of institutional knowledge at exactly the moment when the volume of code being produced is exploding. We are generating more code than ever while retaining less understanding of why it exists.

---

## The Vision: Intent-Based Version Control

The fundamental abstraction of version control has not changed since the 1970s: track which lines were added, removed, or modified between two snapshots of text files. This abstraction was designed for a world where a human author typed every character. In that world, the diff *was* the change. But in AI-assisted development, a single sentence of human intent — "Make the auth system use JWT instead of sessions" — can produce hundreds of lines of coordinated changes across dozens of files. The diff is no longer the change. The intent is the change. The diff is merely an artifact of fulfilling it.

It is time to build version control around this reality.

### Intent as the primary unit

In the current model, a developer makes changes, then writes a commit message summarizing what they did. The commit message is an afterthought — a lossy compression of the developer's actual reasoning, written under the social pressure to be brief. The result is a history full of messages like "fix bug", "update auth", and "WIP".

In intent-based version control, the process inverts. The unit of work begins with a declared intent: a natural-language statement of what the developer wants to accomplish and why. The system records this intent before any code changes. As the human and AI collaborate to fulfill the intent — discussing tradeoffs, exploring approaches, writing and revising code — every step is captured as part of a single, coherent unit of change.

Consider a concrete example. A developer opens their editor and types: "Refactor the payment module to support multiple currency providers instead of being hardcoded to Stripe." The system creates an intent record. Over the next twenty minutes, the developer and AI discuss whether to use a strategy pattern or a plugin architecture, settle on an interface design, generate the provider abstraction, migrate the existing Stripe code into a provider implementation, and update the tests. All of this — the reasoning, the rejected alternatives, the final code changes across fourteen files — lives together as a single intent. The "commit message" was written first. The conversation is the commit body. The diff is an appendix.

### Semantic change tracking

Line-based diffs are the wrong level of abstraction for understanding what changed in a codebase. A developer reviewing a pull request does not actually care that lines 47 through 93 of `auth.py` were replaced with lines 47 through 112. They care that a function called `authenticate()` was added, that the `UserSession` class was removed, and that the return type of `getUser()` changed from `User | null` to `Result<User>`.

Semantic change tracking operates at this structural level. Instead of reporting that 847 lines changed across 23 files — a diff that makes reviewers' eyes glaze over — the system reports a structured summary: three functions were added, one class was removed, two function signatures were modified, a new database migration was created, and the API schema was extended with two new endpoints. Each of these semantic changes links back to the specific lines, but the reviewer starts with meaning, not text.

This is not merely a cosmetic improvement. Semantic tracking makes entirely new operations possible. Merge conflicts can be detected and often resolved at the structural level: if one branch adds a method to a class and another branch renames that class, the system understands both changes and can compose them, rather than dumping a conflict marker into a file and hoping the developer sorts it out. Code review assignments can be routed based on what *kind* of change occurred — schema changes go to the database team, API surface changes trigger a review from the platform group — without relying on file-path heuristics.

### Change layers and perspectives

A single intent produces changes that can be understood at multiple layers of abstraction, and different audiences need different layers. An intent-based system makes this explicit by offering three perspectives on every change:

**The intent layer** shows the goal: "Support multiple currency providers." This is what a project manager or a future developer skimming the history needs. It answers the question *what were we trying to accomplish?*

**The semantic layer** shows the structural impact: "Added interface `CurrencyProvider` in `payments/providers.py`. Extracted class `StripeProvider` implementing `CurrencyProvider`. Modified `PaymentService` constructor to accept a `CurrencyProvider` parameter. Added 12 test cases for provider abstraction." This is where most code review happens. It answers the question *what actually changed in the architecture?*

**The diff layer** shows the raw file changes, line by line. This layer exists for auditing, debugging, and the cases where exact text matters — a security review of a cryptographic implementation, for instance, or tracking down a subtle whitespace issue.

Reviewers choose their depth. A tech lead reviewing a routine feature might never leave the semantic layer. An auditor investigating a production incident can drill from the intent ("Fix race condition in session handling") through the semantic summary ("Added mutex lock in `SessionManager.acquire()`") down to the exact diff to verify correctness. The information is always there; the system simply stops forcing everyone to start at the bottom.

### Conversation as first-class history

Every developer has had the experience of encountering a baffling piece of code, running `git blame`, finding a commit from eighteen months ago with the message "refactor," and learning nothing. The reasoning is gone. In the current model, the *why* behind code changes exists only in the developer's memory, and perhaps in a Slack thread that has long since scrolled into oblivion.

Intent-based version control treats the human-AI conversation as a first-class part of the historical record. When a developer asks the AI "why did we add this retry logic to the payment flow?", the system can surface the original conversation: the developer described intermittent failures from the payment gateway, the AI suggested exponential backoff with jitter, the developer asked about idempotency concerns, and together they settled on a design that included an idempotency key. This entire exchange is as much a part of the version history as the code itself.

This transforms the codebase from a snapshot of *what* the code is into a narrative of *how and why it became what it is*. Onboarding a new team member no longer means reading documentation that is perpetually out of date. It means exploring a living history where every significant decision is accompanied by its rationale.

### Continuous versioning

The traditional commit is a manual act that interrupts the flow of work. Developers must decide when to commit, what to include, and how to message it. The result is predictable: some developers commit too infrequently, producing monolithic changesets that are impossible to review. Others commit too frequently, producing a history cluttered with "WIP" and "fix typo" messages. Squash merges attempt to paper over this by collapsing the mess into a single commit, destroying useful intermediate history in the process.

Continuous versioning eliminates this friction entirely. The system captures state automatically and persistently as the developer works — every meaningful edit, every AI generation, every test run. There are no forgotten changes, no "I forgot to commit before switching branches" disasters. The raw stream of captured state is not the history; it is the raw material from which history is made.

When the developer and AI reach a natural stopping point — a feature works, a bug is fixed, a refactor is complete — they "crystallize" a checkpoint from the continuous stream. The AI proposes a boundary: "It looks like the work on JWT migration is complete. The changes since the last checkpoint touch the auth module, the session middleware, and the test suite. Want to crystallize this as a checkpoint?" The developer agrees, adjusts if needed, and the intent is sealed with its full context. The result is a history that is both complete (nothing is lost) and meaningful (the checkpoints reflect real milestones, not arbitrary save points).

This model also changes how developers think about experimentation. There is no cost to trying something and abandoning it, because the system has already captured the state before the experiment began. There is no need to create a branch "just in case." The continuous stream contains every path taken and not taken, and the crystallized checkpoints mark which paths were chosen.

---

Intent-based version control does not discard the line-level diff. It subsumes it. The diff becomes one layer in a richer structure that also captures meaning, intent, reasoning, and narrative. The result is a version history that is not merely a ledger of text changes but a comprehensible record of how a codebase evolved and why — legible to humans, navigable by AI, and faithful to the way software is actually built today.

---

## Human Oversight: Regaining Transparency and Ownership

The promise of AI-assisted development is velocity. The peril is loss of understanding. When a developer asks an AI agent to "refactor the authentication system for the new SSO provider," the result might touch 40 files across six modules in a single commit. The developer reviews the pull request, scrolls through 1,200 lines of diff, and — if they are honest — approves it based on a spot check and a passing test suite. This is not review. This is ceremony. And it is the default mode of operation in most AI-augmented teams today.

A version control system built for the AI age must treat human oversight not as a bottleneck to be minimized, but as a structural guarantee to be preserved. The question is not whether humans should review AI-generated changes — it is how the system can make that review meaningful rather than performative.

### The review crisis

The traditional pull request was designed for human-scale changes: a few files, a coherent narrative, a reviewer who could hold the full context in working memory. AI-generated changes violate every assumption that model was built on. They are large, structurally distributed, and internally consistent in ways that make it difficult to spot subtle errors. A misnamed variable is easy to catch. A correctly implemented but subtly wrong business rule buried in file 31 of 40 is not.

The result is predictable. Studies of code review behavior consistently show that review quality degrades sharply as diff size increases. When reviewers face AI-generated diffs of hundreds of lines, they default to heuristics: "Do the tests pass? Does the CI pipeline agree? Did the AI seem confident?" These are not unreasonable questions, but they are insufficient. A new system must make large-scale AI changes reviewable by design, not by heroic effort.

### Impact-first review

The fundamental error of current review tooling is that it presents changes at the wrong level of abstraction. A diff shows you *what text changed*. What a reviewer actually needs to know is *what behavior changed*.

An impact-first review surface would answer concrete questions before showing a single line of code. What API contracts changed? Which public function signatures were modified? What test cases now produce different results? Which modules have new dependencies they did not have before? What error paths were added or removed?

Consider the SSO refactor example. Instead of 40 file diffs, the reviewer sees a structured summary: "Three API endpoints changed their authentication middleware. Two new database migrations were added. The `UserSession` interface gained two fields. Four tests were updated to reflect new session semantics, and one new integration test was added. No changes were made to billing, payments, or audit logging." The reviewer can now make a judgment about whether the scope is appropriate, then drill into the specific migration files or the changed interface definition — not because they are reviewing every line, but because they are investigating areas of concern.

### Trust scoring and provenance

Not all AI-generated code carries the same risk. A formatting change the AI made with high confidence is different from a security-sensitive authorization check the AI flagged as uncertain. The version control system should track this distinction explicitly.

Every change should carry provenance metadata: was this line written by a human, generated by an AI with explicit human instruction, or autonomously produced by an AI agent acting on a broader goal? What was the AI's reported confidence? Did the human review this specific change, or did they approve the commit as a whole?

This creates a trust gradient rather than a binary. When a new developer opens a file six months later, they can see not just who changed it but how much human judgment was involved. A function marked as "AI-generated, high confidence, human-approved at commit level" carries a different weight than one marked as "human-written, manually tested." Neither is inherently better, but the distinction matters for debugging, auditing, and regulatory compliance.

### Ownership without micromanagement

Developers already think in terms of ownership — "that's the payments team's module" — but current tools encode this weakly, if at all. In an AI-augmented workflow, ownership must become a first-class constraint.

An ownership map defines zones of authority. The billing module might be marked as "human-only: require explicit approval for any change." The utility library might be "AI-autonomous: the agent can refactor freely within test constraints." The API layer might be "shared: AI can propose, human must approve." These are not social conventions — they are rules the version control system enforces. An AI agent that attempts to modify a human-only zone without authorization has its change rejected at the system level, not at the review stage.

This eliminates an entire class of oversight failures. The developer does not need to notice that the AI touched the billing module during a broad refactor. The system prevents it from happening in the first place.

### Explainable history

`git blame` answers the question "who changed this line and when." It does not answer the question that actually matters: *why*.

In an AI-driven workflow, the chain of intent is longer and more complex. A line of code might exist because a developer asked the AI to fix a performance issue, which the AI diagnosed as a missing database index, which required restructuring a query builder, which required changing a function signature in a shared utility. The final change — a modified function signature — is incomprehensible without the full chain.

An explainable history links every change to its originating intent, the conversation that produced it, and the reasoning the AI followed. A new developer investigating a confusing function signature can trace it back to the performance ticket, read the AI's diagnosis, and understand why this particular approach was chosen over alternatives. This is not documentation bolted on after the fact. It is history that carries its own explanation, recorded automatically as a natural byproduct of the AI interaction.

### Regression and drift detection

Individual AI changes may each be correct in isolation while collectively drifting from the system's architectural intent. Over ten refactoring sessions, an AI might gradually shift a codebase from the event-driven architecture documented in the design spec toward a request-response pattern, simply because each local change was easier to implement that way.

The version control system should maintain a model of stated design goals — architectural patterns, module boundaries, dependency rules, performance budgets — and continuously compare the evolving codebase against them. When cumulative changes push a metric past a threshold (dependency depth increases by 40%, a module's public API surface doubles, circular dependencies appear), the system raises a drift alert. This is not a linting rule. It is a structural check that operates across the full history of changes, catching the slow erosion that no individual review would flag.

The goal across all of these mechanisms is the same: to keep humans genuinely informed rather than nominally responsible. Oversight that depends on human vigilance alone will fail at scale. Oversight that is embedded in the system's structure — in how changes are presented, tracked, constrained, explained, and monitored — can scale with the AI's capabilities rather than against them.

---

## Collaboration: Humans, AIs, and Teams

Version control has always been, at its core, a collaboration tool. But it was designed for a world where every contributor is a human typing code into a text editor, committing a handful of changes a day, and resolving the occasional merge conflict over a misplaced semicolon. That world is ending. When AI agents become first-class participants in development — writing code, reviewing it, refactoring it, and even coordinating with each other — the entire collaboration model needs to be rethought from the ground up.

### Multi-agent coordination

Consider a scenario that is already emerging: a team spins up three AI agents simultaneously — one to refactor the authentication module, one to add rate limiting to the API layer, and one to migrate the database schema. All three touch overlapping files. In Git, this produces a nightmare of merge conflicts, because Git operates at the level of text. It sees two different rewrites of the same file and throws up its hands.

A next-generation version control system needs **semantic merge** — the ability to understand what a change *means*, not just what text it altered. If Agent A rewrites a function to extract a helper method, and Agent B rewrites the same function to add error handling, a semantic merge engine can recognize that these intents are compatible and compose them. The function gets both the extracted helper and the error handling. This requires the VCS to operate on an abstract syntax tree (or even a richer intent graph) rather than raw text diffs.

Practically, this means agents would declare their *intent* before beginning work — "I am refactoring `AuthService.validate()` to separate token parsing from validation logic" — and the system would detect conflicts at the intent level. Two agents both restructuring the same module? Flag it before a single line is written. Two agents working on orthogonal concerns in the same file? Let them proceed in parallel, and reconcile automatically.

### Human-AI pair versioning

Today's commit is a lossy snapshot. It captures *what* changed, but not *why*, and certainly not the conversation that led to the change. When a developer works with an AI assistant, the real unit of work is the **session**: the prompt the human wrote, the code the AI proposed, the parts the human accepted, the parts they rejected, the follow-up refinements. All of this context is currently discarded.

A collaboration-aware VCS would preserve this decision trail as a first-class artifact. Imagine a "session object" attached to each commit that records: the original intent ("make this endpoint idempotent"), the AI's initial proposal, the human's modifications ("no, don't use Redis for this — use a database-level constraint"), and the final accepted code. Six months later, when someone asks "why does this endpoint use a unique constraint instead of a cache?", the answer is right there in the session history — not buried in a Slack thread or lost entirely.

### Team dynamics at scale

A team of five developers, each working with their own AI, can easily generate in a single day what used to take a week. The rate of change explodes. Traditional review queues buckle under the volume, and merge conflicts multiply combinatorially.

The new VCS needs mechanisms to absorb this velocity. **Intent-level conflict detection** is one: before any code is generated, developers register their goals ("I'm adding caching to the user service," "I'm refactoring the user service to use dependency injection"), and the system warns them of collisions upfront. **AI-summarized changelogs** are another: instead of reviewing 40 diffs, a reviewer reads a concise summary — "This set of changes adds retry logic to all external API calls, with exponential backoff and a circuit breaker. No behavioral changes to happy-path flows." The reviewer can then drill into the actual code only where their judgment is needed.

### Knowledge propagation

When a developer's AI discovers through trial and error that a particular module is brittle — say, the payment processing pipeline breaks if events arrive out of order — that knowledge is currently trapped in one person's context window. It vanishes when the session ends.

A collaboration-aware VCS would serve as a **shared knowledge layer**. Annotations like "this module is order-sensitive; do not parallelize event handlers" would attach to the code itself and propagate to every agent that touches it. When another developer's AI proposes a refactor that introduces concurrent event processing in that module, the system would surface the warning automatically. The codebase accumulates institutional knowledge in a machine-readable form, not just in the heads of senior engineers.

### Role-based views of history

A single history cannot serve all audiences in raw form. A developer debugging a regression wants to see **semantic diffs** — what behavioral changes were introduced, not that 200 lines were reformatted. A security auditor wants every AI-generated code block **flagged and attributable**, with the ability to filter history to show only AI-authored changes. A product manager wants **intent-level progress** — "authentication module: complete; rate limiting: in progress; database migration: blocked" — without wading through commits. A new team member wants the **narrative arc** — why the codebase evolved the way it did, told as a story of decisions rather than a log of patches.

This demands a version history that stores rich, structured metadata and supports multiple projections over the same underlying data. One history, many lenses. The raw commit log becomes an implementation detail; the real interface is a queryable, role-aware view of how the software came to be.

---

The shift is fundamental. Version control stops being a passive ledger of text changes and becomes an active collaboration platform — one that understands intent, preserves decision-making context, mediates between agents, propagates knowledge, and presents itself differently to every stakeholder who needs it.

---

## Technical Sketch: What This System Could Look Like

Reimagining version control for the AI age is not about discarding git's hard-won insights — content-addressable storage, distributed operation, cryptographic integrity — but about building a fundamentally richer data model on top of those foundations. What follows is a concrete technical sketch of what such a system, which we will call **aig** (AI-git), might look like.

### Core data model: From blobs to intent graphs

Git's data model is elegant in its simplicity: blobs, trees, commits, and refs. Every commit is a snapshot of a file tree with a pointer to its parent. This model is agnostic about *why* a change was made, *who suggested it* (human or AI), and *what goal* it served. It captures state, not intent.

An AI-native system replaces this with three interlocking layers:

**1. The Intent Graph.** The top-level structure is a directed acyclic graph of *intents* — goal-oriented nodes that represent what a developer (or an AI agent) set out to accomplish. Each intent node contains:

- A natural-language description of the goal (e.g., "Add JWT-based authentication to the API layer")
- Links to one or more *conversation threads* — the back-and-forth between human and AI that shaped the implementation
- Links to the *semantic changes* the intent produced
- Parent/child relationships to other intents (decomposition: "Add JWT auth" decomposes into "generate tokens," "validate tokens," "add middleware")

This graph is the primary navigational structure. Where `git log` shows a linear (or branching) sequence of snapshots, `aig history --intent` shows a tree of goals and sub-goals.

**2. Semantic AST Snapshots.** Below the intent layer, changes are recorded not as text diffs but as transformations on abstract syntax trees. When a developer adds a function, the system records "function node `validate_token` added to module `auth` with parameters `(token: str, secret: str)` returning `Claims`." When an AI refactors a loop into a list comprehension, the system records the structural transformation, not the line-level diff.

This has several advantages. Semantic diffs are immune to formatting noise. They enable queries like "show me every function signature that changed in the last sprint" without parsing diffs. And they provide the foundation for AI-powered review: a reviewer can examine changes at the level of *what the code does*, not merely *what text moved*.

**3. Continuous State Stream with Crystallization Points.** Rather than requiring explicit commits, the system maintains a continuous append-only stream of fine-grained state changes — every keystroke, every AI suggestion accepted or rejected, every test run. This stream is ephemeral by default and garbage-collected aggressively. But developers can *crystallize* moments from the stream into named checkpoints:

```
aig checkpoint "JWT token generation working, tests passing"
```

A crystallization point is roughly analogous to a git commit, but it carries with it a window into the stream — the conversation context, the false starts, the discarded approaches. A checkpoint is not just a snapshot; it is a *summary with receipts*.

### Storage and efficiency

Git's content-addressable store is remarkably space-efficient: identical blobs are stored once, packfiles use delta compression, and the entire history of the Linux kernel fits in a few gigabytes. An AI-native system must match this efficiency while storing far richer metadata.

The key insight is that structured representations compress better than text. Storing code as **AST plus formatting rules** (via a deterministic formatter like `rustfmt` or `black`) is often smaller than storing raw source, because the formatting rules are shared across the entire repository and the AST contains only semantic content. Conversations between human and AI are stored as **structured data** — role-tagged message sequences with references to code ranges (file, AST node path, byte offset) rather than copy-pasted code blocks. A single AI conversation that references 20 code locations stores 20 pointers, not 20 duplicated snippets.

For indexing and search, the system uses **LLM-generated summaries** as a first-class part of the data model. Each intent node, each checkpoint, and each conversation thread has an auto-generated summary that serves as a searchable index entry. These summaries are themselves content-addressed: if the underlying data has not changed, the summary is not regenerated. The result is a system where `aig search "authentication bypass vulnerability"` can return relevant intents, conversations, and code locations — not just grep matches.

Delta compression extends naturally to ASTs. Two versions of a syntax tree share most of their nodes; only the changed subtrees need to be stored. This is analogous to git's packfile deltas but operates at a semantically meaningful granularity.

### The CLI and daily workflow

The day-to-day experience should feel familiar to git users while surfacing the richer data model. Here is a plausible session:

```bash
# Start a tracked session with a declared intent
aig session start "Add JWT authentication to /api endpoints"

# Work proceeds — code is written, AI is consulted, tests are run.
# The system records the state stream in the background.

# Crystallize a checkpoint when something meaningful is reached
aig checkpoint "JWT token generation and validation implemented"

# Continue working...
aig checkpoint "Auth middleware integrated, all routes protected"

# Review changes at the semantic level before sharing
aig review --level=semantic
# Output:
#   Module auth (new)
#     + function generate_token(user_id: str, secret: str) -> str
#     + function validate_token(token: str, secret: str) -> Claims
#   Module middleware (modified)
#     + function require_auth(handler: Callable) -> Callable
#   Module routes (modified)
#     ~ 4 route handlers wrapped with require_auth

# View intent-level history
aig history --intent
# Output:
#   [a3f1] Add JWT authentication to /api endpoints
#     [b7c2] ├── JWT token generation and validation
#     [d4e8] └── Auth middleware integration

# Ask why a specific line exists
aig why src/auth.py:42
# Output:
#   Line 42: token = jwt.encode(payload, secret, algorithm="HS256")
#   Intent: "Add JWT authentication to /api endpoints" [a3f1]
#   Conversation: AI suggested HS256 over RS256 for simplicity
#     in single-service deployment (message #14 in session)
#   Checkpoint: "JWT token generation and validation implemented" [b7c2]

# Merge intent-aware: the system understands that two developers
# worked on different intents and can merge at the semantic level
aig merge feature/rate-limiting --strategy=semantic

# Push to a shared remote (which is git-compatible under the hood)
aig push origin main
```

The `aig why` command is perhaps the most transformative. Today, `git blame` tells you *who* changed a line and *when*. `aig why` tells you *what goal it served*, *what conversation produced it*, and *what alternatives were considered*. This is the difference between an audit trail and an explanation.

### Migration and coexistence

No version control system can succeed by demanding a clean break from git. The adoption path must be incremental.

**Layer 1: Git as storage backend.** The simplest starting point is to use git as the underlying storage layer and add the intent/semantic/conversation layers as supplementary metadata. Checkpoints map to git commits. Intent graphs and conversation logs are stored in a parallel data structure (a `.aig/` directory, analogous to `.git/`, or as git notes and custom refs). Any aig repository is simultaneously a valid git repository. Collaborators who do not use aig see normal commits; collaborators who do see the richer context.

**Layer 2: Bidirectional sync.** An `aig import` command can ingest an existing git history and use LLM analysis to retroactively infer intents, group related commits, and generate summaries. This is lossy — the original conversations are gone — but it provides a meaningful upgrade for existing repositories. In the other direction, `aig export --format=git` produces a clean git history suitable for consumption by any standard tool.

**Layer 3: Gradual team adoption.** Teams can adopt aig incrementally. One developer starts using `aig session` and `aig checkpoint` while others continue using raw git. The aig metadata enriches the shared history without disrupting anyone else's workflow. As more team members adopt the tool, the intent graph becomes more complete and more useful — a positive network effect.

### What exists today

Several existing tools and research efforts point in this direction, though none yet offer the integrated vision described above:

- **Semantic diff tools** like Difftastic and GumTree already parse code into ASTs and produce structural diffs rather than line-level ones. They prove the feasibility of AST-based change tracking.
- **GitButler** and the now-sunset **Sturdy** explored richer-than-git workflows, with GitButler offering virtual branches and continuous working-directory awareness — a step toward the "continuous state stream" concept.
- **Graphite** focuses on stacked pull requests and streamlined review workflows, demonstrating that developers will adopt tools that layer on top of git when the UX payoff is clear.
- **AI-powered code review tools** like CodeRabbit, Ellipsis, and GitHub Copilot for Pull Requests are beginning to perform semantic analysis of changes at review time — generating exactly the kind of summaries that could be stored as first-class metadata.
- **Cursor, Windsurf, and similar AI-native editors** already maintain conversation history linked to code changes; they simply do not persist it into version control.
- **Research into provenance tracking** in computational notebooks (e.g., noWorkflow, ProvBook) explores how to capture the full lineage of a result, including the iterative process that produced it — closely analogous to what an intent graph captures for general software.

The pieces exist in isolation. The opportunity is to compose them into a coherent system that treats AI-era development as a first-class concern rather than an afterthought bolted onto a 2005-era data model.

---

## Conclusion: From Ledger to Living History

Git transformed software development by making version control distributed, fast, and reliable. It solved the problems of its era brilliantly. But its era — one where humans typed every line, reviewed every diff, and carried the reasoning in their heads — is giving way to something fundamentally different.

The problems are clear:

- **Commits** are either too large to review or artificially split into fiction
- **Diffs** show text changes but obscure meaning
- **Authorship** is binary in a world of human-AI collaboration
- **Branching** adds ceremony without value when AI collapses timelines
- **History** captures what changed but loses why

The solution is not to patch git with better commit messages or smarter diff tools. It is to rethink the foundational abstraction. Version control should track **intent**, not just text. It should store **conversations**, not just diffs. It should present changes at **multiple levels of abstraction** — intent, semantic, and textual — and let each audience choose their depth. It should make AI changes **reviewable by design** through impact-first review, trust scoring, and ownership maps. It should support **multi-agent collaboration** through semantic merging and intent-level conflict detection. And it should serve as a **shared knowledge layer** that accumulates institutional understanding rather than discarding it.

The working name for this vision is **aig** — a system that:

| Git | aig |
|---|---|
| Line-based diffs | Semantic AST changes |
| Manual commits | Continuous versioning with crystallization |
| Commit messages (afterthought) | Intents (declared upfront) |
| No conversation history | Conversations as first-class history |
| Binary authorship | Trust gradients and provenance |
| Text-level merge | Semantic merge |
| `git blame` (who/when) | `aig why` (intent/reasoning/alternatives) |
| File-path ownership (CODEOWNERS) | Semantic ownership maps |
| One view for everyone | Role-based perspectives |

The path forward is incremental. Build on git, not against it. Layer intent and semantic tracking on top. Let teams adopt gradually. Make the richer metadata available to those who want it without disrupting those who don't.

Git gave us reliable version control. The AI age demands *comprehensible* version control — a system where the history of a codebase is not just an audit trail but a living narrative that humans can understand, trust, and learn from, no matter how fast the machines write the code.

---

*This document was produced collaboratively by a human and five parallel AI research agents, each exploring a different facet of the problem — a fitting demonstration of the very workflow it describes.*
