(function() {var implementors = {};
implementors["alloc_compose"] = [{"text":"impl&lt;A, const SIZE:&nbsp;usize&gt; Send for Chunk&lt;A, SIZE&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A: Send,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Send for Null","synthetic":true,"types":[]},{"text":"impl&lt;A, C&gt; Send for Proxy&lt;A, C&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A: Send,<br>&nbsp;&nbsp;&nbsp;&nbsp;C: Send,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl !Send for RawRegion","synthetic":true,"types":[]},{"text":"impl !Send for RawSharedRegion","synthetic":true,"types":[]},{"text":"impl !Send for RawIntrusiveRegion","synthetic":true,"types":[]},{"text":"impl&lt;'mem&gt; !Send for Region&lt;'mem&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'mem&gt; !Send for SharedRegion&lt;'mem&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'mem&gt; !Send for IntrusiveRegion&lt;'mem&gt;","synthetic":true,"types":[]},{"text":"impl Send for Counter","synthetic":true,"types":[]},{"text":"impl Send for AtomicCounter","synthetic":true,"types":[]},{"text":"impl Send for AllocInitFilter","synthetic":true,"types":[]},{"text":"impl Send for ReallocPlacementFilter","synthetic":true,"types":[]},{"text":"impl Send for ResultFilter","synthetic":true,"types":[]},{"text":"impl Send for FilteredCounter","synthetic":true,"types":[]},{"text":"impl Send for FilteredAtomicCounter","synthetic":true,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()