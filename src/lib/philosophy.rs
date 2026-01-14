//! Compiler philosophy and design notes.
//!
//! This module documents the guiding principles behind the compiler, including the distinction between expressions and statements, operand vs variable locations, and code generation philosophy.

//! Philosophy of the Compiler.
//!
//! ### 1. Expression vs Statement
//!
//! ###### **Expressions**:
//!   - Are **transient computations**.
//!   - Operate in terms of [`crate::backend::mem::OperandLocation`]:
//!     - [`crate::backend::mem::OperandLocation::REG`] → value stored in a CPU register.
//!     - [`crate::backend::mem::OperandLocation::STK`] → value in the stack.
//!     - [`crate::backend::mem::OperandLocation::IMM`] → immediate literal (temporary, ephemeral).
//!   - Can be combined, lowered, and evaluated.
//!   - Never commit `IMM` to symbol table.
//!
//! ###### **Statements**:
//!   - Anchor values to actual **storage locations**.
//!   - Operate in terms of [`crate::backend::mem::VariableLocation`]:
//!     - [`crate::backend::mem::VariableLocation::REG`] → variable stored in a register.
//!     - [`crate::backend::mem::VariableLocation::STK`] → variable stored on stack.
//!   - Convert [`crate::backend::mem::OperandLocation`] from expressions into storage locations.
//!   - Assignments (`Assign`) and declarations (`Let`) handle [`crate::backend::mem::OperandLocation::IMM`] by allocating a temporary register.
//!
//! ### 2. Variable Design
//!
//! - [`crate::backend::mem::Variable`] struct only tracks **where the variable lives**, never literal values.
//! - [`crate::backend::mem::OperandLocation::IMM`] is **never part of a variable's location**.
//! - Symbol table maps identifiers → [`crate::backend::mem::VariableLocation`].
//! - Helper conversion from [`crate::backend::mem::VariableLocation`] → [`crate::backend::mem::OperandLocation`] for codegen is allowed.
//!
//! ### 3. Operand Philosophy
//!
//! - [`crate::backend::mem::OperandLocation`] is the **playground for expressions**.
//! - Can contain [`crate::backend::mem::OperandLocation::IMM`] because expressions can be literals.
//! - Expressions return [`crate::backend::mem::OperandLocation`] to allow flexible lowering.
//! - Statements decide how to handle operands and move them into storage.
//!
//! ### 4. Code Generation Philosophy
//!
//! - Codegen works with **operand locations** first.
//! - When lowering expressions, handle [`crate::backend::mem::OperandLocation::IMM`] by allocating temp registers as needed.
//! - After evaluation, move results into variable storage ([`crate::backend::mem::VariableLocation`]).
//! - Free temporary registers immediately after use.
//!
//! ### 5. Semantic Rules
//!
//! - [`crate::backend::mem::OperandLocation::IMM`] is ephemeral and never persisted in the symbol table.
//! - Variables only point to real locations ([`crate::backend::mem::VariableLocation::REG`] or [`crate::backend::mem::VariableLocation::STK`]).
//! - Expressions are composable; statements give them meaning by anchoring them to storage.
//! - All lowering and register allocation respects the distinction between **transient operand** and **persistent storage**.
//!
//! ### 6. Naming Conventions
//!
//! - [`crate::backend::mem::OperandLocation`] → used in expressions and codegen, may include `IMM`.
//! - [`crate::backend::mem::VariableLocation`] → used in symbol table for persistent storage, excludes `IMM`.
//! - This separation prevents bugs and enforces clarity in the compiler design.
//!
//! ### 7. Summary
//!
//! - **Expressions** = playground, may include [`crate::backend::mem::OperandLocation::IMM`].
//! - **Statements** = storage anchor, never `IMM`.
//! - **Variables** = symbolic reference to storage, clean separation from temporary computations.
//! - **Codegen** = works with operands first, converts to storage as needed, frees temps immediately.
//! - Philosophy encourages **clarity, safety, and proper register management**.

// Dummy item so the module compiles
pub struct Philosophy;

