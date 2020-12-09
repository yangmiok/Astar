(function() {var implementors = {};
implementors["ovmi"] = [{"text":"impl Serialize for PredicateTypeSerializable","synthetic":false,"types":[]},{"text":"impl Serialize for VarTypeSerializable","synthetic":false,"types":[]},{"text":"impl Serialize for CompiledPredicateSerializable","synthetic":false,"types":[]},{"text":"impl Serialize for ConstantVariableSerializable","synthetic":false,"types":[]},{"text":"impl Serialize for IntermediateCompiledPredicateSerializable","synthetic":false,"types":[]},{"text":"impl Serialize for AtomicPropositionOrPlaceholderSerializable","synthetic":false,"types":[]},{"text":"impl Serialize for AtomicPropositionSerializable","synthetic":false,"types":[]},{"text":"impl Serialize for PredicateCallSerializable","synthetic":false,"types":[]},{"text":"impl Serialize for AtomicPredicateCallSerializable","synthetic":false,"types":[]},{"text":"impl Serialize for InputPredicateCallSerializable","synthetic":false,"types":[]},{"text":"impl Serialize for VariablePredicateCallSerializable","synthetic":false,"types":[]},{"text":"impl Serialize for CompiledPredicateCallSerializable","synthetic":false,"types":[]},{"text":"impl Serialize for CompiledInputSerializable","synthetic":false,"types":[]},{"text":"impl Serialize for ConstantInputSerializable","synthetic":false,"types":[]},{"text":"impl Serialize for LabelInputSerializable","synthetic":false,"types":[]},{"text":"impl Serialize for NormalInputSerializable","synthetic":false,"types":[]},{"text":"impl Serialize for VariableInputSerializable","synthetic":false,"types":[]},{"text":"impl Serialize for SelfInputSerializable","synthetic":false,"types":[]},{"text":"impl Serialize for LogicalConnectiveSerializable","synthetic":false,"types":[]}];
implementors["pallet_contract_operator"] = [{"text":"impl Serialize for DefaultParameters","synthetic":false,"types":[]}];
implementors["pallet_dapps_staking"] = [{"text":"impl Serialize for StakingParameters","synthetic":false,"types":[]},{"text":"impl Serialize for GenesisConfig","synthetic":false,"types":[]}];
implementors["pallet_operator_trading"] = [{"text":"impl Serialize for OfferState","synthetic":false,"types":[]},{"text":"impl&lt;AccountId, Balance, Moment&gt; Serialize for Offer&lt;AccountId, Balance, Moment&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;AccountId: Serialize,<br>&nbsp;&nbsp;&nbsp;&nbsp;Balance: Serialize,<br>&nbsp;&nbsp;&nbsp;&nbsp;Moment: Serialize,&nbsp;</span>","synthetic":false,"types":[]}];
implementors["pallet_ovm"] = [{"text":"impl Serialize for Schedule","synthetic":false,"types":[]},{"text":"impl Serialize for GenesisConfig <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Schedule: Serialize,&nbsp;</span>","synthetic":false,"types":[]}];
implementors["pallet_plasm_lockdrop"] = [{"text":"impl&lt;T:&nbsp;Trait&gt; Serialize for GenesisConfig&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Vec&lt;T::AuthorityId&gt;: Serialize,<br>&nbsp;&nbsp;&nbsp;&nbsp;Perbill: Serialize,<br>&nbsp;&nbsp;&nbsp;&nbsp;(T::DollarRate, T::DollarRate): Serialize,<br>&nbsp;&nbsp;&nbsp;&nbsp;AuthorityVote: Serialize,<br>&nbsp;&nbsp;&nbsp;&nbsp;AuthorityVote: Serialize,<br>&nbsp;&nbsp;&nbsp;&nbsp;(T::BlockNumber, T::BlockNumber): Serialize,&nbsp;</span>","synthetic":false,"types":[]}];
implementors["pallet_plasm_rewards"] = [{"text":"impl Serialize for GenesisConfig <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;u32: Serialize,<br>&nbsp;&nbsp;&nbsp;&nbsp;Forcing: Serialize,&nbsp;</span>","synthetic":false,"types":[]}];
implementors["pallet_plasm_validator"] = [{"text":"impl&lt;T:&nbsp;Trait&gt; Serialize for GenesisConfig&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Vec&lt;T::AccountId&gt;: Serialize,&nbsp;</span>","synthetic":false,"types":[]}];
implementors["plasm_cli"] = [{"text":"impl Serialize for Extensions","synthetic":false,"types":[]}];
implementors["plasm_runtime"] = [{"text":"impl Serialize for SessionKeys","synthetic":false,"types":[]},{"text":"impl Serialize for GenesisConfig","synthetic":false,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()