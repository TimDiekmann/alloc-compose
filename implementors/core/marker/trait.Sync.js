(function() {var implementors = {};
implementors["alloc_compose"] = [{"text":"impl&lt;A, const SIZE:&nbsp;usize&gt; Sync for Chunk&lt;A, SIZE&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A: Sync,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;Primary, Secondary&gt; Sync for Fallback&lt;Primary, Secondary&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Primary: Sync,<br>&nbsp;&nbsp;&nbsp;&nbsp;Secondary: Sync,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Sync for Null","synthetic":true,"types":[]},{"text":"impl&lt;A, C&gt; Sync for Proxy&lt;A, C&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A: Sync,<br>&nbsp;&nbsp;&nbsp;&nbsp;C: Sync,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl !Sync for RawRegion","synthetic":true,"types":[]},{"text":"impl !Sync for RawSharedRegion","synthetic":true,"types":[]},{"text":"impl !Sync for RawIntrusiveRegion","synthetic":true,"types":[]},{"text":"impl&lt;'mem&gt; !Sync for Region&lt;'mem&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'mem&gt; !Sync for SharedRegion&lt;'mem&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'mem&gt; !Sync for IntrusiveRegion&lt;'mem&gt;","synthetic":true,"types":[]},{"text":"impl !Sync for Counter","synthetic":true,"types":[]},{"text":"impl Sync for AtomicCounter","synthetic":true,"types":[]},{"text":"impl Sync for AllocInitFilter","synthetic":true,"types":[]},{"text":"impl Sync for ReallocPlacementFilter","synthetic":true,"types":[]},{"text":"impl Sync for ResultFilter","synthetic":true,"types":[]},{"text":"impl !Sync for FilteredCounter","synthetic":true,"types":[]},{"text":"impl Sync for FilteredAtomicCounter","synthetic":true,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()