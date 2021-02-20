(function() {var implementors = {};
implementors["full_node"] = [{"text":"impl StructuralPartialEq for BlocksRequestId","synthetic":false,"types":[]}];
implementors["smoldot"] = [{"text":"impl StructuralPartialEq for SlotClaim","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for TransactionValidityError","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for InvalidTransaction","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for UnknownTransaction","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for DispatchError","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for NodeIndex","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for RequestId","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for SourceId","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for ExecHint","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for Signature","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for ValueType","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for CoreVersion","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; StructuralPartialEq for CoreVersionRef&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; StructuralPartialEq for AuraConsensusLogRef&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for AuraConsensusLog","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; StructuralPartialEq for AuraAuthorityRef&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for AuraAuthority","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for AuraPreDigest","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; StructuralPartialEq for BabeConsensusLogRef&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for BabeConsensusLog","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; StructuralPartialEq for BabeNextEpochRef&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for BabeNextEpoch","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; StructuralPartialEq for BabeAuthorityRef&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for BabeAuthority","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for BabeNextConfig","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for BabeAllowedSlots","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; StructuralPartialEq for BabePreDigestRef&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for BabeSecondaryPlainPreDigest","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; StructuralPartialEq for GrandpaConsensusLogRef&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for GrandpaConsensusLog","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; StructuralPartialEq for GrandpaScheduledChangeRef&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for GrandpaScheduledChange","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; StructuralPartialEq for GrandpaAuthorityRef&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for GrandpaAuthority","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; StructuralPartialEq for DigestItemRef&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for ChangesTrieSignal","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for ChangesTrieConfiguration","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for ConnectionId","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for SubstreamId","synthetic":false,"types":[]},{"text":"impl&lt;I, P&gt; StructuralPartialEq for Config&lt;I, P&gt;","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for SubstreamId","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for PublicKey","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for SubstreamDirection","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for ConnectionId","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for PendingId","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for ConnectionId","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; StructuralPartialEq for MetadataRef&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; StructuralPartialEq for ModuleMetadataRef&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; StructuralPartialEq for StorageMetadataRef&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; StructuralPartialEq for StorageEntryMetadataRef&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for StorageEntryModifier","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; StructuralPartialEq for StorageEntryTypeRef&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for StorageHasher","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; StructuralPartialEq for FunctionMetadataRef&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; StructuralPartialEq for FunctionArgumentMetadataRef&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; StructuralPartialEq for EventMetadataRef&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; StructuralPartialEq for ModuleConstantMetadataRef&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; StructuralPartialEq for ErrorMetadataRef&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; StructuralPartialEq for ExtrinsicMetadataRef&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; StructuralPartialEq for BlockAnnouncesHandshakeRef&lt;'a&gt;","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for Role","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for BlocksRequestConfig","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for BlocksRequestDirection","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for BlocksRequestFields","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for BlocksRequestConfigStart","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for BlockData","synthetic":false,"types":[]},{"text":"impl&lt;TKeysIter&gt; StructuralPartialEq for StorageProofRequestConfig&lt;TKeysIter&gt;","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for PendingId","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for ConnectionId","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for Nibble","synthetic":false,"types":[]},{"text":"impl StructuralPartialEq for NodeIndex","synthetic":false,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()