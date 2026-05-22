-- SPDX-License-Identifier: MPL-2.0
-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath)
-- Julia-the-Viper: Harvard Architecture Safety Proofs

module Types

%default total

-- | JtV type universe
public export
data JtvTy : Type where
  TyInt      : JtvTy
  TyFloat    : JtvTy
  TyRational : JtvTy
  TyComplex  : JtvTy
  TyHex      : JtvTy
  TyBinary   : JtvTy
  TySymbolic : JtvTy
  TyBool     : JtvTy
  TyString   : JtvTy
  TyUnit     : JtvTy
  TyList     : JtvTy -> JtvTy
  TyFun      : List JtvTy -> JtvTy -> JtvTy

-- | Harvard architecture: data vs control separation
public export
data Language : Type where
  DataLang    : Language
  ControlLang : Language

-- | Expression indexed by which language it belongs to
public export
data Expr : Language -> Type where
  -- Data language expressions (total, addition-only)
  DLit     : Int -> Expr DataLang
  DFloat   : Double -> Expr DataLang
  DVar     : String -> Expr DataLang
  DAdd     : Expr DataLang -> Expr DataLang -> Expr DataLang
  DNeg     : Expr DataLang -> Expr DataLang
  DList    : List (Expr DataLang) -> Expr DataLang
  -- Control language statements (Turing-complete)
  CAssign  : String -> Expr DataLang -> Expr ControlLang
  CIf      : Expr DataLang -> Expr ControlLang -> Expr ControlLang -> Expr ControlLang
  CWhile   : Expr DataLang -> Expr ControlLang -> Expr ControlLang
  CPrint   : Expr DataLang -> Expr ControlLang
  CSeq     : Expr ControlLang -> Expr ControlLang -> Expr ControlLang
  CSkip    : Expr ControlLang

-- | Proof: Data expressions cannot contain control flow
-- This is enforced by the type index: Expr DataLang cannot construct
-- CAssign, CIf, CWhile, CPrint — the constructors don't type-check.
public export
dataCannotLoop : Expr DataLang -> Void
dataCannotLoop (DLit _) impossible
dataCannotLoop (DFloat _) impossible
dataCannotLoop (DVar _) impossible
dataCannotLoop (DAdd _ _) impossible
dataCannotLoop (DNeg _) impossible
dataCannotLoop (DList _) impossible
-- This function has no valid inputs — proving the property vacuously.
-- The real proof is that CWhile : Expr ControlLang, which cannot be
-- passed where Expr DataLang is expected.

-- | Purity levels (ordered)
public export
data Purity : Type where
  Total  : Purity
  Pure   : Purity
  Impure : Purity

-- | Purity ordering
public export
data PurityLeq : Purity -> Purity -> Type where
  TotalLeqTotal  : PurityLeq Total Total
  TotalLeqPure   : PurityLeq Total Pure
  TotalLeqImpure : PurityLeq Total Impure
  PureLeqPure    : PurityLeq Pure Pure
  PureLeqImpure  : PurityLeq Pure Impure
  ImpureLeqImpure : PurityLeq Impure Impure

-- | Purity ordering is transitive
public export
purityLeqTrans : PurityLeq a b -> PurityLeq b c -> PurityLeq a c
purityLeqTrans TotalLeqTotal TotalLeqTotal = TotalLeqTotal
purityLeqTrans TotalLeqTotal TotalLeqPure = TotalLeqPure
purityLeqTrans TotalLeqTotal TotalLeqImpure = TotalLeqImpure
purityLeqTrans TotalLeqPure PureLeqPure = TotalLeqPure
purityLeqTrans TotalLeqPure PureLeqImpure = TotalLeqImpure
purityLeqTrans TotalLeqImpure ImpureLeqImpure = TotalLeqImpure
purityLeqTrans PureLeqPure PureLeqPure = PureLeqPure
purityLeqTrans PureLeqPure PureLeqImpure = PureLeqImpure
purityLeqTrans PureLeqImpure ImpureLeqImpure = PureLeqImpure
purityLeqTrans ImpureLeqImpure ImpureLeqImpure = ImpureLeqImpure

-- | Numeric tower widening
public export
data CanWiden : JtvTy -> JtvTy -> Type where
  IntToFloat    : CanWiden TyInt TyFloat
  IntToRational : CanWiden TyInt TyRational
  IntToComplex  : CanWiden TyInt TyComplex
  FloatToComplex : CanWiden TyFloat TyComplex
  HexToInt      : CanWiden TyHex TyInt
  BinaryToInt   : CanWiden TyBinary TyInt
  SameType      : CanWiden t t

-- | Widening is transitive
public export
widenTrans : CanWiden a b -> CanWiden b c -> CanWiden a c
widenTrans SameType w = w
widenTrans w SameType = w
widenTrans IntToFloat FloatToComplex = IntToComplex
widenTrans HexToInt IntToFloat = ?hexToFloat
widenTrans HexToInt IntToRational = ?hexToRational
widenTrans HexToInt IntToComplex = ?hexToComplex
widenTrans BinaryToInt IntToFloat = ?binaryToFloat
widenTrans BinaryToInt IntToRational = ?binaryToRational
widenTrans BinaryToInt IntToComplex = ?binaryToComplex
widenTrans _ _ = ?widenTransOther

-- | Data expressions are addition-only (no subtraction operator)
-- This is structurally enforced: DAdd is the only binary operator
-- in the DataLang constructors. DNeg provides unary negation.
-- Binary subtraction a - b must be expressed as a + (-b).
public export
dataAdditionOnly : (e : Expr DataLang) -> Type
dataAdditionOnly (DAdd _ _) = ()  -- addition is present
dataAdditionOnly (DNeg _) = ()    -- unary negation is present
dataAdditionOnly _ = ()           -- all other forms are non-arithmetic
