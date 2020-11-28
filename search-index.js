var searchIndex = JSON.parse('{\
"alloc_compose":{"doc":"Test Status Coverage Status Docs master Docs.rs Crates.io…","i":[[0,"region","alloc_compose","Stack-based allocators with user-provided memory",null,null],[0,"raw","alloc_compose::region","Region implementations which are not bound by a lifetime.",null,null],[3,"RawRegion","alloc_compose::region::raw","A stack allocator over an user-defined region of memory.",null,null],[11,"new","","Creates a new region from the given memory block.",0,[[["nonnull",3]]]],[3,"RawSharedRegion","","",null,null],[11,"new","","Creates a new region from the given memory block.",1,[[["nonnull",3]]]],[3,"RawIntrusiveRegion","","An intrusive region allocator, which stores the current…",null,null],[11,"new","","Creates a new region from the given memory block.",2,[[["nonnull",3]]]],[3,"Region","alloc_compose::region","A stack allocator over an user-defined region of memory.",null,null],[11,"new","","Creates a new region from the given memory block.",3,[[]]],[3,"SharedRegion","","A clonable region allocator based on `Rc`.",null,null],[11,"new","","Creates a new region from the given memory block.",4,[[]]],[3,"IntrusiveRegion","","An intrusive region allocator, which stores the current…",null,null],[11,"new","","Creates a new region from the given memory block.",5,[[]]],[0,"stats","alloc_compose","Structures to collect allocator statistics.",null,null],[3,"Counter","alloc_compose::stats","A primitive counter for collectiong statistics.",null,null],[3,"AtomicCounter","","An atomic counter for collectiong statistics which can be…",null,null],[11,"num_allocs","","Returns the number of `alloc` calls.",6,[[]]],[11,"num_deallocs","","Returns the number of `dealloc` calls.",6,[[]]],[11,"num_grows","","Returns the number of `grow` calls.",6,[[]]],[11,"num_shrinks","","Returns the number of `shrink` calls.",6,[[]]],[11,"num_owns","","Returns the number of `owns` calls.",6,[[]]],[11,"num_allocs","","Returns the number of `alloc` calls.",7,[[]]],[11,"num_deallocs","","Returns the number of `dealloc` calls.",7,[[]]],[11,"num_grows","","Returns the number of `grow` calls.",7,[[]]],[11,"num_shrinks","","Returns the number of `shrink` calls.",7,[[]]],[11,"num_owns","","Returns the number of `owns` calls.",7,[[]]],[4,"AllocInitFilter","","",null,null],[13,"None","","",8,null],[13,"Uninitialized","","",8,null],[13,"Zeroed","","",8,null],[4,"ReallocPlacementFilter","","",null,null],[13,"None","","",9,null],[13,"MayMove","","",9,null],[13,"InPlace","","",9,null],[4,"ResultFilter","","",null,null],[13,"None","","",10,null],[13,"Ok","","",10,null],[13,"Err","","",10,null],[3,"FilteredCounter","","A counter for collectiong and filtering statistics.",null,null],[3,"FilteredAtomicCounter","","An atomic counter for collectiong and filtering statistics…",null,null],[11,"num_allocates","","Returns the total number of `alloc` calls.",11,[[]]],[11,"num_allocates_filter","","Returns the filtered number of `alloc` calls.",11,[[]]],[11,"num_deallocates","","Returns the total number of `dealloc` calls.",11,[[]]],[11,"num_grows","","Returns the total number of `grow` calls.",11,[[]]],[11,"num_grows_filter","","Returns the filtered number of `grow` calls.",11,[[]]],[11,"num_shrinks","","Returns the total number of `shrink` calls.",11,[[]]],[11,"num_shrinks_filter","","Returns the filtered number of `shrink` calls.",11,[[["reallocplacementfilter",4]]]],[11,"num_owns","","Returns the total number of `owns` calls.",11,[[]]],[11,"num_owns_filter","","Returns the filtered number of `owns` calls.",11,[[]]],[11,"num_allocates","","Returns the total number of `alloc` calls.",12,[[]]],[11,"num_allocates_filter","","Returns the filtered number of `alloc` calls.",12,[[]]],[11,"num_deallocates","","Returns the total number of `dealloc` calls.",12,[[]]],[11,"num_grows","","Returns the total number of `grow` calls.",12,[[]]],[11,"num_grows_filter","","Returns the filtered number of `grow` calls.",12,[[]]],[11,"num_shrinks","","Returns the total number of `shrink` calls.",12,[[]]],[11,"num_shrinks_filter","","Returns the filtered number of `shrink` calls.",12,[[["reallocplacementfilter",4]]]],[11,"num_owns","","Returns the total number of `owns` calls.",12,[[]]],[11,"num_owns_filter","","Returns the filtered number of `owns` calls.",12,[[]]],[8,"CallbackRef","alloc_compose","Backend for the `Proxy` allocator.",null,null],[11,"before_allocate","","Called before `alloc` was invoked.",13,[[["layout",3]]]],[11,"after_allocate","","Called after `alloc` was invoked.",13,[[["allocerror",3],["nonnull",3],["result",4],["layout",3]]]],[11,"before_allocate_zeroed","","Called before `alloc_zeroed` was invoked.",13,[[["layout",3]]]],[11,"after_allocate_zeroed","","Called after `alloc_zeroed` was invoked.",13,[[["allocerror",3],["nonnull",3],["result",4],["layout",3]]]],[11,"before_allocate_all","","Called before `allocate_all` was invoked.",13,[[]]],[11,"after_allocate_all","","Called after `allocate_all` was invoked.",13,[[["nonnull",3],["result",4],["allocerror",3]]]],[11,"before_allocate_all_zeroed","","Called before `allocate_all_zeroed` was invoked.",13,[[]]],[11,"after_allocate_all_zeroed","","Called after `allocate_all_zeroed` was invoked.",13,[[["nonnull",3],["result",4],["allocerror",3]]]],[11,"before_deallocate","","Called before `dealloc` was invoked.",13,[[["layout",3],["nonnull",3]]]],[11,"after_deallocate","","Called after `dealloc` was invoked.",13,[[["layout",3],["nonnull",3]]]],[11,"before_deallocate_all","","Called before `deallocate_all` was invoked.",13,[[]]],[11,"after_deallocate_all","","Called after `deallocate_all` was invoked.",13,[[]]],[11,"before_grow","","Called before `grow` was invoked.",13,[[["layout",3],["nonnull",3]]]],[11,"after_grow","","Called after `grow` was invoked.",13,[[["allocerror",3],["nonnull",3],["nonnull",3],["result",4],["layout",3]]]],[11,"before_grow_zeroed","","Called before `grow_zeroed` was invoked.",13,[[["layout",3],["nonnull",3]]]],[11,"after_grow_zeroed","","Called after `grow_zeroed` was invoked.",13,[[["allocerror",3],["nonnull",3],["nonnull",3],["result",4],["layout",3]]]],[11,"before_grow_in_place","","Called before `grow_in_place` was invoked.",13,[[["layout",3],["nonnull",3]]]],[11,"after_grow_in_place","","Called after `grow_in_place` was invoked.",13,[[["allocerror",3],["nonnull",3],["layout",3],["result",4]]]],[11,"before_grow_in_place_zeroed","","Called before `grow_in_place_zeroed` was invoked.",13,[[["layout",3],["nonnull",3]]]],[11,"after_grow_in_place_zeroed","","Called after `grow_in_place_zeroed` was invoked.",13,[[["allocerror",3],["nonnull",3],["layout",3],["result",4]]]],[11,"before_shrink","","Called before `shrink` was invoked.",13,[[["layout",3],["nonnull",3]]]],[11,"after_shrink","","Called after `shrink` was invoked.",13,[[["allocerror",3],["nonnull",3],["nonnull",3],["result",4],["layout",3]]]],[11,"before_shrink_in_place","","Called before `shrink_in_place` was invoked.",13,[[["layout",3],["nonnull",3]]]],[11,"after_shrink_in_place","","Called after `shrink_in_place` was invoked.",13,[[["allocerror",3],["nonnull",3],["layout",3],["result",4]]]],[11,"before_owns","","Called before `owns` was invoked.",13,[[]]],[11,"after_owns","","Called after `owns` was invoked.",13,[[]]],[11,"by_ref","","Creates a \\\"by reference\\\" adaptor for this instance of…",13,[[]]],[3,"Chunk","","Allocate memory with a multiple size of the provided chunk…",null,null],[12,"0","","",14,null],[3,"Null","","An emphatically empty implementation of `AllocRef`.",null,null],[3,"Proxy","","Calls the provided callbacks when invoking methods on…",null,null],[12,"alloc","","",15,null],[12,"callbacks","","",15,null],[8,"AllocateAll","","Extends `AllocRef` for allocating or deallocating all…",null,null],[10,"allocate_all","","Attempts to allocate all of the memory the allocator can…",16,[[],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"allocate_all_zeroed","","Behaves like `alloc_all`, but also ensures that the…",16,[[],[["nonnull",3],["result",4],["allocerror",3]]]],[10,"deallocate_all","","Deallocates all the memory the allocator had allocated.",16,[[]]],[10,"capacity","","Returns the total capacity available in this allocator.",16,[[]]],[10,"capacity_left","","Returns the free capacity left for allocating.",16,[[]]],[11,"is_empty","","Returns if the allocator is currently not holding memory.",16,[[]]],[11,"is_full","","Returns if the allocator has no more capacity left.",16,[[]]],[8,"ReallocateInPlace","","Extends `AllocRef` to support growing and shrinking in…",null,null],[10,"grow_in_place","","Attempts to extend the memory block.",17,[[["layout",3],["nonnull",3]],[["allocerror",3],["result",4]]]],[10,"grow_in_place_zeroed","","Behaves like `grow_in_place`, but also ensures that the…",17,[[["layout",3],["nonnull",3]],[["allocerror",3],["result",4]]]],[10,"shrink_in_place","","Attempts to shrink the memory block.",17,[[["layout",3],["nonnull",3]],[["allocerror",3],["result",4]]]],[8,"Owns","","Trait to determine if a given memory block is owned by an…",null,null],[10,"owns","","Returns if the allocator owns the passed memory.",18,[[["nonnull",3]]]],[11,"from","","",14,[[]]],[11,"borrow","","",14,[[]]],[11,"borrow_mut","","",14,[[]]],[11,"try_from","","",14,[[],["result",4]]],[11,"into","","",14,[[]]],[11,"try_into","","",14,[[],["result",4]]],[11,"type_id","","",14,[[],["typeid",3]]],[11,"to_owned","","",14,[[]]],[11,"clone_into","","",14,[[]]],[11,"from","","",19,[[]]],[11,"borrow","","",19,[[]]],[11,"borrow_mut","","",19,[[]]],[11,"try_from","","",19,[[],["result",4]]],[11,"into","","",19,[[]]],[11,"try_into","","",19,[[],["result",4]]],[11,"type_id","","",19,[[],["typeid",3]]],[11,"to_owned","","",19,[[]]],[11,"clone_into","","",19,[[]]],[11,"from","","",15,[[]]],[11,"borrow","","",15,[[]]],[11,"borrow_mut","","",15,[[]]],[11,"try_from","","",15,[[],["result",4]]],[11,"into","","",15,[[]]],[11,"try_into","","",15,[[],["result",4]]],[11,"type_id","","",15,[[],["typeid",3]]],[11,"to_owned","","",15,[[]]],[11,"clone_into","","",15,[[]]],[11,"from","alloc_compose::region::raw","",0,[[]]],[11,"borrow","","",0,[[]]],[11,"borrow_mut","","",0,[[]]],[11,"try_from","","",0,[[],["result",4]]],[11,"into","","",0,[[]]],[11,"try_into","","",0,[[],["result",4]]],[11,"type_id","","",0,[[],["typeid",3]]],[11,"from","","",1,[[]]],[11,"borrow","","",1,[[]]],[11,"borrow_mut","","",1,[[]]],[11,"try_from","","",1,[[],["result",4]]],[11,"into","","",1,[[]]],[11,"try_into","","",1,[[],["result",4]]],[11,"type_id","","",1,[[],["typeid",3]]],[11,"to_owned","","",1,[[]]],[11,"clone_into","","",1,[[]]],[11,"from","","",2,[[]]],[11,"borrow","","",2,[[]]],[11,"borrow_mut","","",2,[[]]],[11,"try_from","","",2,[[],["result",4]]],[11,"into","","",2,[[]]],[11,"try_into","","",2,[[],["result",4]]],[11,"type_id","","",2,[[],["typeid",3]]],[11,"to_owned","","",2,[[]]],[11,"clone_into","","",2,[[]]],[11,"from","alloc_compose::region","",3,[[]]],[11,"borrow","","",3,[[]]],[11,"borrow_mut","","",3,[[]]],[11,"try_from","","",3,[[],["result",4]]],[11,"into","","",3,[[]]],[11,"try_into","","",3,[[],["result",4]]],[11,"type_id","","",3,[[],["typeid",3]]],[11,"from","","",4,[[]]],[11,"borrow","","",4,[[]]],[11,"borrow_mut","","",4,[[]]],[11,"try_from","","",4,[[],["result",4]]],[11,"into","","",4,[[]]],[11,"try_into","","",4,[[],["result",4]]],[11,"type_id","","",4,[[],["typeid",3]]],[11,"to_owned","","",4,[[]]],[11,"clone_into","","",4,[[]]],[11,"from","","",5,[[]]],[11,"borrow","","",5,[[]]],[11,"borrow_mut","","",5,[[]]],[11,"try_from","","",5,[[],["result",4]]],[11,"into","","",5,[[]]],[11,"try_into","","",5,[[],["result",4]]],[11,"type_id","","",5,[[],["typeid",3]]],[11,"to_owned","","",5,[[]]],[11,"clone_into","","",5,[[]]],[11,"from","alloc_compose::stats","",6,[[]]],[11,"borrow","","",6,[[]]],[11,"borrow_mut","","",6,[[]]],[11,"try_from","","",6,[[],["result",4]]],[11,"into","","",6,[[]]],[11,"try_into","","",6,[[],["result",4]]],[11,"type_id","","",6,[[],["typeid",3]]],[11,"from","","",7,[[]]],[11,"borrow","","",7,[[]]],[11,"borrow_mut","","",7,[[]]],[11,"try_from","","",7,[[],["result",4]]],[11,"into","","",7,[[]]],[11,"try_into","","",7,[[],["result",4]]],[11,"type_id","","",7,[[],["typeid",3]]],[11,"from","","",8,[[]]],[11,"borrow","","",8,[[]]],[11,"borrow_mut","","",8,[[]]],[11,"try_from","","",8,[[],["result",4]]],[11,"into","","",8,[[]]],[11,"try_into","","",8,[[],["result",4]]],[11,"type_id","","",8,[[],["typeid",3]]],[11,"to_owned","","",8,[[]]],[11,"clone_into","","",8,[[]]],[11,"from","","",9,[[]]],[11,"borrow","","",9,[[]]],[11,"borrow_mut","","",9,[[]]],[11,"try_from","","",9,[[],["result",4]]],[11,"into","","",9,[[]]],[11,"try_into","","",9,[[],["result",4]]],[11,"type_id","","",9,[[],["typeid",3]]],[11,"to_owned","","",9,[[]]],[11,"clone_into","","",9,[[]]],[11,"from","","",10,[[]]],[11,"borrow","","",10,[[]]],[11,"borrow_mut","","",10,[[]]],[11,"try_from","","",10,[[],["result",4]]],[11,"into","","",10,[[]]],[11,"try_into","","",10,[[],["result",4]]],[11,"type_id","","",10,[[],["typeid",3]]],[11,"to_owned","","",10,[[]]],[11,"clone_into","","",10,[[]]],[11,"from","","",11,[[]]],[11,"borrow","","",11,[[]]],[11,"borrow_mut","","",11,[[]]],[11,"try_from","","",11,[[],["result",4]]],[11,"into","","",11,[[]]],[11,"try_into","","",11,[[],["result",4]]],[11,"type_id","","",11,[[],["typeid",3]]],[11,"from","","",12,[[]]],[11,"borrow","","",12,[[]]],[11,"borrow_mut","","",12,[[]]],[11,"try_from","","",12,[[],["result",4]]],[11,"into","","",12,[[]]],[11,"try_into","","",12,[[],["result",4]]],[11,"type_id","","",12,[[],["typeid",3]]],[11,"after_allocate","","",6,[[["allocerror",3],["nonnull",3],["result",4],["layout",3]]]],[11,"after_allocate_zeroed","","",6,[[["allocerror",3],["nonnull",3],["result",4],["layout",3]]]],[11,"after_allocate_all","","",6,[[["nonnull",3],["result",4],["allocerror",3]]]],[11,"after_allocate_all_zeroed","","",6,[[["nonnull",3],["result",4],["allocerror",3]]]],[11,"before_deallocate","","",6,[[["layout",3],["nonnull",3]]]],[11,"before_deallocate_all","","",6,[[]]],[11,"after_grow","","",6,[[["allocerror",3],["nonnull",3],["nonnull",3],["result",4],["layout",3]]]],[11,"after_grow_zeroed","","",6,[[["allocerror",3],["nonnull",3],["nonnull",3],["result",4],["layout",3]]]],[11,"after_grow_in_place","","",6,[[["allocerror",3],["nonnull",3],["layout",3],["result",4]]]],[11,"after_grow_in_place_zeroed","","",6,[[["allocerror",3],["nonnull",3],["layout",3],["result",4]]]],[11,"after_shrink","","",6,[[["allocerror",3],["nonnull",3],["nonnull",3],["result",4],["layout",3]]]],[11,"after_shrink_in_place","","",6,[[["allocerror",3],["nonnull",3],["layout",3],["result",4]]]],[11,"after_owns","","",6,[[]]],[11,"after_allocate","","",7,[[["allocerror",3],["nonnull",3],["result",4],["layout",3]]]],[11,"after_allocate_zeroed","","",7,[[["allocerror",3],["nonnull",3],["result",4],["layout",3]]]],[11,"after_allocate_all","","",7,[[["nonnull",3],["result",4],["allocerror",3]]]],[11,"after_allocate_all_zeroed","","",7,[[["nonnull",3],["result",4],["allocerror",3]]]],[11,"before_deallocate","","",7,[[["layout",3],["nonnull",3]]]],[11,"before_deallocate_all","","",7,[[]]],[11,"after_grow","","",7,[[["allocerror",3],["nonnull",3],["nonnull",3],["result",4],["layout",3]]]],[11,"after_grow_zeroed","","",7,[[["allocerror",3],["nonnull",3],["nonnull",3],["result",4],["layout",3]]]],[11,"after_grow_in_place","","",7,[[["allocerror",3],["nonnull",3],["layout",3],["result",4]]]],[11,"after_grow_in_place_zeroed","","",7,[[["allocerror",3],["nonnull",3],["layout",3],["result",4]]]],[11,"after_shrink","","",7,[[["allocerror",3],["nonnull",3],["nonnull",3],["result",4],["layout",3]]]],[11,"after_shrink_in_place","","",7,[[["allocerror",3],["nonnull",3],["layout",3],["result",4]]]],[11,"after_owns","","",7,[[]]],[11,"after_allocate","","",11,[[["allocerror",3],["nonnull",3],["result",4],["layout",3]]]],[11,"after_allocate_zeroed","","",11,[[["allocerror",3],["nonnull",3],["result",4],["layout",3]]]],[11,"after_allocate_all","","",11,[[["nonnull",3],["result",4],["allocerror",3]]]],[11,"after_allocate_all_zeroed","","",11,[[["nonnull",3],["result",4],["allocerror",3]]]],[11,"before_deallocate","","",11,[[["layout",3],["nonnull",3]]]],[11,"before_deallocate_all","","",11,[[]]],[11,"after_grow","","",11,[[["allocerror",3],["nonnull",3],["nonnull",3],["result",4],["layout",3]]]],[11,"after_grow_zeroed","","",11,[[["allocerror",3],["nonnull",3],["nonnull",3],["result",4],["layout",3]]]],[11,"after_grow_in_place","","",11,[[["allocerror",3],["nonnull",3],["layout",3],["result",4]]]],[11,"after_grow_in_place_zeroed","","",11,[[["allocerror",3],["nonnull",3],["layout",3],["result",4]]]],[11,"after_shrink","","",11,[[["allocerror",3],["nonnull",3],["nonnull",3],["result",4],["layout",3]]]],[11,"after_shrink_in_place","","",11,[[["allocerror",3],["nonnull",3],["layout",3],["result",4]]]],[11,"after_owns","","",11,[[]]],[11,"after_allocate","","",12,[[["allocerror",3],["nonnull",3],["result",4],["layout",3]]]],[11,"after_allocate_zeroed","","",12,[[["allocerror",3],["nonnull",3],["result",4],["layout",3]]]],[11,"after_allocate_all","","",12,[[["nonnull",3],["result",4],["allocerror",3]]]],[11,"after_allocate_all_zeroed","","",12,[[["nonnull",3],["result",4],["allocerror",3]]]],[11,"before_deallocate","","",12,[[["layout",3],["nonnull",3]]]],[11,"before_deallocate_all","","",12,[[]]],[11,"after_grow","","",12,[[["allocerror",3],["nonnull",3],["nonnull",3],["result",4],["layout",3]]]],[11,"after_grow_zeroed","","",12,[[["allocerror",3],["nonnull",3],["nonnull",3],["result",4],["layout",3]]]],[11,"after_grow_in_place","","",12,[[["allocerror",3],["nonnull",3],["layout",3],["result",4]]]],[11,"after_grow_in_place_zeroed","","",12,[[["allocerror",3],["nonnull",3],["layout",3],["result",4]]]],[11,"after_shrink","","",12,[[["allocerror",3],["nonnull",3],["nonnull",3],["result",4],["layout",3]]]],[11,"after_shrink_in_place","","",12,[[["allocerror",3],["nonnull",3],["layout",3],["result",4]]]],[11,"after_owns","","",12,[[]]],[11,"allocate_all","alloc_compose","",19,[[],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"allocate_all_zeroed","","",19,[[],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"deallocate_all","","",19,[[]]],[11,"capacity","","",19,[[]]],[11,"capacity_left","","",19,[[]]],[11,"allocate_all","","",15,[[],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"allocate_all_zeroed","","",15,[[],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"deallocate_all","","",15,[[]]],[11,"capacity","","",15,[[]]],[11,"capacity_left","","",15,[[]]],[11,"is_empty","","",15,[[]]],[11,"is_full","","",15,[[]]],[11,"allocate_all","alloc_compose::region::raw","",0,[[],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"deallocate_all","","",0,[[]]],[11,"capacity","","",0,[[]]],[11,"capacity_left","","",0,[[]]],[11,"allocate_all","","",1,[[],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"deallocate_all","","",1,[[]]],[11,"capacity","","",1,[[]]],[11,"capacity_left","","",1,[[]]],[11,"allocate_all","","",2,[[],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"deallocate_all","","",2,[[]]],[11,"capacity","","",2,[[]]],[11,"capacity_left","","",2,[[]]],[11,"allocate_all","alloc_compose::region","",3,[[],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"allocate_all_zeroed","","",3,[[],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"deallocate_all","","",3,[[]]],[11,"capacity","","",3,[[]]],[11,"capacity_left","","",3,[[]]],[11,"allocate_all","","",4,[[],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"allocate_all_zeroed","","",4,[[],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"deallocate_all","","",4,[[]]],[11,"capacity","","",4,[[]]],[11,"capacity_left","","",4,[[]]],[11,"allocate_all","","",5,[[],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"allocate_all_zeroed","","",5,[[],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"deallocate_all","","",5,[[]]],[11,"capacity","","",5,[[]]],[11,"capacity_left","","",5,[[]]],[11,"grow_in_place","alloc_compose","",14,[[["layout",3],["nonnull",3]],[["allocerror",3],["result",4]]]],[11,"grow_in_place_zeroed","","",14,[[["layout",3],["nonnull",3]],[["allocerror",3],["result",4]]]],[11,"shrink_in_place","","",14,[[["layout",3],["nonnull",3]],[["allocerror",3],["result",4]]]],[11,"grow_in_place","","",14,[[["layout",3],["nonnull",3]],[["allocerror",3],["result",4]]]],[11,"grow_in_place_zeroed","","",14,[[["layout",3],["nonnull",3]],[["allocerror",3],["result",4]]]],[11,"shrink_in_place","","",14,[[["layout",3],["nonnull",3]],[["allocerror",3],["result",4]]]],[11,"grow_in_place","","Must not be called, as allocation always fails.",19,[[["layout",3],["nonnull",3]],[["allocerror",3],["result",4]]]],[11,"grow_in_place_zeroed","","Must not be called, as allocation always fails.",19,[[["layout",3],["nonnull",3]],[["allocerror",3],["result",4]]]],[11,"shrink_in_place","","Must not be called, as allocation always fails.",19,[[["layout",3],["nonnull",3]],[["allocerror",3],["result",4]]]],[11,"grow_in_place","","",15,[[["layout",3],["nonnull",3]],[["allocerror",3],["result",4]]]],[11,"grow_in_place_zeroed","","",15,[[["layout",3],["nonnull",3]],[["allocerror",3],["result",4]]]],[11,"shrink_in_place","","",15,[[["layout",3],["nonnull",3]],[["allocerror",3],["result",4]]]],[11,"owns","","",14,[[["nonnull",3]]]],[11,"owns","","Will always return `false.",19,[[["nonnull",3]]]],[11,"owns","","",15,[[["nonnull",3]]]],[11,"owns","alloc_compose::region::raw","",0,[[["nonnull",3]]]],[11,"owns","","",1,[[["nonnull",3]]]],[11,"owns","","",2,[[["nonnull",3]]]],[11,"owns","alloc_compose::region","",3,[[["nonnull",3]]]],[11,"owns","","",4,[[["nonnull",3]]]],[11,"owns","","",5,[[["nonnull",3]]]],[11,"from","alloc_compose::stats","",10,[[]]],[11,"fmt","alloc_compose","",14,[[["formatter",3]],["result",6]]],[11,"fmt","","",19,[[["formatter",3]],["result",6]]],[11,"fmt","","",15,[[["formatter",3]],["result",6]]],[11,"fmt","alloc_compose::region::raw","",0,[[["formatter",3]],["result",6]]],[11,"fmt","","",1,[[["formatter",3]],["result",6]]],[11,"fmt","","",2,[[["formatter",3]],["result",6]]],[11,"fmt","alloc_compose::stats","",6,[[["formatter",3]],["result",6]]],[11,"fmt","","",7,[[["formatter",3]],["result",6]]],[11,"fmt","","",8,[[["formatter",3]],["result",6]]],[11,"fmt","","",9,[[["formatter",3]],["result",6]]],[11,"fmt","","",10,[[["formatter",3]],["result",6]]],[11,"fmt","","",11,[[["formatter",3]],["result",6]]],[11,"fmt","","",12,[[["formatter",3]],["result",6]]],[11,"eq","alloc_compose","",14,[[["chunk",3]]]],[11,"ne","","",14,[[["chunk",3]]]],[11,"eq","","",15,[[["proxy",3]]]],[11,"ne","","",15,[[["proxy",3]]]],[11,"eq","alloc_compose::region::raw","",0,[[]]],[11,"eq","","",1,[[]]],[11,"eq","","",2,[[]]],[11,"eq","alloc_compose::region","",3,[[]]],[11,"eq","","",3,[[["rawregion",3]]]],[11,"eq","alloc_compose::region::raw","",0,[[["region",3]]]],[11,"eq","alloc_compose::region","",4,[[]]],[11,"eq","","",4,[[["rawsharedregion",3]]]],[11,"eq","alloc_compose::region::raw","",1,[[["sharedregion",3]]]],[11,"eq","alloc_compose::region","",5,[[]]],[11,"eq","","",5,[[["rawintrusiveregion",3]]]],[11,"eq","alloc_compose::region::raw","",2,[[["intrusiveregion",3]]]],[11,"eq","alloc_compose::stats","",6,[[["counter",3]]]],[11,"ne","","",6,[[["counter",3]]]],[11,"eq","","",6,[[["atomiccounter",3]]]],[11,"eq","","",7,[[]]],[11,"eq","","",7,[[["counter",3]]]],[11,"eq","","",8,[[["allocinitfilter",4]]]],[11,"eq","","",9,[[["reallocplacementfilter",4]]]],[11,"eq","","",10,[[["resultfilter",4]]]],[11,"eq","","",11,[[["filteredcounter",3]]]],[11,"ne","","",11,[[["filteredcounter",3]]]],[11,"eq","","",11,[[["filteredatomiccounter",3]]]],[11,"eq","","",12,[[]]],[11,"eq","","",12,[[["filteredcounter",3]]]],[11,"cmp","alloc_compose","",15,[[["proxy",3]],["ordering",4]]],[11,"partial_cmp","","",15,[[["proxy",3]],[["ordering",4],["option",4]]]],[11,"lt","","",15,[[["proxy",3]]]],[11,"le","","",15,[[["proxy",3]]]],[11,"gt","","",15,[[["proxy",3]]]],[11,"ge","","",15,[[["proxy",3]]]],[11,"hash","","",15,[[]]],[11,"clone","","",14,[[],["chunk",3]]],[11,"clone","","",19,[[],["null",3]]],[11,"clone","","",15,[[],["proxy",3]]],[11,"clone","alloc_compose::region::raw","",1,[[],["rawsharedregion",3]]],[11,"clone","","",2,[[],["rawintrusiveregion",3]]],[11,"clone","alloc_compose::region","",4,[[],["sharedregion",3]]],[11,"clone","","",5,[[],["intrusiveregion",3]]],[11,"clone","alloc_compose::stats","",8,[[],["allocinitfilter",4]]],[11,"clone","","",9,[[],["reallocplacementfilter",4]]],[11,"clone","","",10,[[],["resultfilter",4]]],[11,"default","alloc_compose","",14,[[],["chunk",3]]],[11,"default","alloc_compose::stats","",6,[[],["counter",3]]],[11,"default","","",7,[[],["atomiccounter",3]]],[11,"default","","",11,[[],["filteredcounter",3]]],[11,"default","","",12,[[],["filteredatomiccounter",3]]],[11,"alloc","alloc_compose","",19,[[["layout",3]]]],[11,"dealloc","","",19,[[["layout",3]]]],[11,"alloc_zeroed","","",19,[[["layout",3]]]],[11,"realloc","","",19,[[["layout",3]]]],[11,"alloc","alloc_compose::region::raw","",0,[[["layout",3]]]],[11,"dealloc","","",0,[[["layout",3]]]],[11,"alloc_zeroed","","",0,[[["layout",3]]]],[11,"realloc","","",0,[[["layout",3]]]],[11,"alloc","","",1,[[["layout",3]]]],[11,"dealloc","","",1,[[["layout",3]]]],[11,"alloc_zeroed","","",1,[[["layout",3]]]],[11,"realloc","","",1,[[["layout",3]]]],[11,"alloc","","",2,[[["layout",3]]]],[11,"dealloc","","",2,[[["layout",3]]]],[11,"alloc_zeroed","","",2,[[["layout",3]]]],[11,"realloc","","",2,[[["layout",3]]]],[11,"alloc","alloc_compose::region","",3,[[["layout",3]]]],[11,"dealloc","","",3,[[["layout",3]]]],[11,"alloc_zeroed","","",3,[[["layout",3]]]],[11,"realloc","","",3,[[["layout",3]]]],[11,"alloc","","",4,[[["layout",3]]]],[11,"dealloc","","",4,[[["layout",3]]]],[11,"alloc_zeroed","","",4,[[["layout",3]]]],[11,"realloc","","",4,[[["layout",3]]]],[11,"alloc","","",5,[[["layout",3]]]],[11,"dealloc","","",5,[[["layout",3]]]],[11,"alloc_zeroed","","",5,[[["layout",3]]]],[11,"realloc","","",5,[[["layout",3]]]],[11,"alloc","alloc_compose","",14,[[["layout",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"alloc_zeroed","","",14,[[["layout",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"grow","","",14,[[["layout",3],["nonnull",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"grow_zeroed","","",14,[[["layout",3],["nonnull",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"shrink","","",14,[[["layout",3],["nonnull",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"dealloc","","",14,[[["layout",3],["nonnull",3]]]],[11,"alloc","","Will always return `Err(AllocErr)`.",19,[[["layout",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"alloc_zeroed","","Will always return `Err(AllocErr)`.",19,[[["layout",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"dealloc","","Must not be called, as allocation always fails.",19,[[["layout",3],["nonnull",3]]]],[11,"grow","","Must not be called, as allocation always fails.",19,[[["layout",3],["nonnull",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"grow_zeroed","","Must not be called, as allocation always fails.",19,[[["layout",3],["nonnull",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"shrink","","Must not be called, as allocation always fails.",19,[[["layout",3],["nonnull",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"alloc","","",15,[[["layout",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"alloc_zeroed","","",15,[[["layout",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"dealloc","","",15,[[["layout",3],["nonnull",3]]]],[11,"grow","","",15,[[["layout",3],["nonnull",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"grow_zeroed","","",15,[[["layout",3],["nonnull",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"shrink","","",15,[[["layout",3],["nonnull",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"alloc","alloc_compose::region::raw","",0,[[["layout",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"dealloc","","",0,[[["layout",3],["nonnull",3]]]],[11,"alloc","","",1,[[["layout",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"dealloc","","",1,[[["layout",3],["nonnull",3]]]],[11,"alloc","","",2,[[["layout",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"dealloc","","",2,[[["layout",3],["nonnull",3]]]],[11,"alloc","alloc_compose::region","",3,[[["layout",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"dealloc","","",3,[[["layout",3],["nonnull",3]]]],[11,"grow","","",3,[[["layout",3],["nonnull",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"grow_zeroed","","",3,[[["layout",3],["nonnull",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"shrink","","",3,[[["layout",3],["nonnull",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"alloc","","",4,[[["layout",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"dealloc","","",4,[[["layout",3],["nonnull",3]]]],[11,"grow","","",4,[[["layout",3],["nonnull",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"grow_zeroed","","",4,[[["layout",3],["nonnull",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"shrink","","",4,[[["layout",3],["nonnull",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"alloc","","",5,[[["layout",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"dealloc","","",5,[[["layout",3],["nonnull",3]]]],[11,"grow","","",5,[[["layout",3],["nonnull",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"grow_zeroed","","",5,[[["layout",3],["nonnull",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"shrink","","",5,[[["layout",3],["nonnull",3]],[["nonnull",3],["result",4],["allocerror",3]]]],[11,"before_allocate","alloc_compose","Called before `alloc` was invoked.",13,[[["layout",3]]]],[11,"after_allocate","","Called after `alloc` was invoked.",13,[[["allocerror",3],["nonnull",3],["result",4],["layout",3]]]],[11,"before_allocate_zeroed","","Called before `alloc_zeroed` was invoked.",13,[[["layout",3]]]],[11,"after_allocate_zeroed","","Called after `alloc_zeroed` was invoked.",13,[[["allocerror",3],["nonnull",3],["result",4],["layout",3]]]],[11,"before_allocate_all","","Called before `allocate_all` was invoked.",13,[[]]],[11,"after_allocate_all","","Called after `allocate_all` was invoked.",13,[[["nonnull",3],["result",4],["allocerror",3]]]],[11,"before_allocate_all_zeroed","","Called before `allocate_all_zeroed` was invoked.",13,[[]]],[11,"after_allocate_all_zeroed","","Called after `allocate_all_zeroed` was invoked.",13,[[["nonnull",3],["result",4],["allocerror",3]]]],[11,"before_deallocate","","Called before `dealloc` was invoked.",13,[[["layout",3],["nonnull",3]]]],[11,"after_deallocate","","Called after `dealloc` was invoked.",13,[[["layout",3],["nonnull",3]]]],[11,"before_deallocate_all","","Called before `deallocate_all` was invoked.",13,[[]]],[11,"after_deallocate_all","","Called after `deallocate_all` was invoked.",13,[[]]],[11,"before_grow","","Called before `grow` was invoked.",13,[[["layout",3],["nonnull",3]]]],[11,"after_grow","","Called after `grow` was invoked.",13,[[["allocerror",3],["nonnull",3],["nonnull",3],["result",4],["layout",3]]]],[11,"before_grow_zeroed","","Called before `grow_zeroed` was invoked.",13,[[["layout",3],["nonnull",3]]]],[11,"after_grow_zeroed","","Called after `grow_zeroed` was invoked.",13,[[["allocerror",3],["nonnull",3],["nonnull",3],["result",4],["layout",3]]]],[11,"before_grow_in_place","","Called before `grow_in_place` was invoked.",13,[[["layout",3],["nonnull",3]]]],[11,"after_grow_in_place","","Called after `grow_in_place` was invoked.",13,[[["allocerror",3],["nonnull",3],["layout",3],["result",4]]]],[11,"before_grow_in_place_zeroed","","Called before `grow_in_place_zeroed` was invoked.",13,[[["layout",3],["nonnull",3]]]],[11,"after_grow_in_place_zeroed","","Called after `grow_in_place_zeroed` was invoked.",13,[[["allocerror",3],["nonnull",3],["layout",3],["result",4]]]],[11,"before_shrink","","Called before `shrink` was invoked.",13,[[["layout",3],["nonnull",3]]]],[11,"after_shrink","","Called after `shrink` was invoked.",13,[[["allocerror",3],["nonnull",3],["nonnull",3],["result",4],["layout",3]]]],[11,"before_shrink_in_place","","Called before `shrink_in_place` was invoked.",13,[[["layout",3],["nonnull",3]]]],[11,"after_shrink_in_place","","Called after `shrink_in_place` was invoked.",13,[[["allocerror",3],["nonnull",3],["layout",3],["result",4]]]],[11,"before_owns","","Called before `owns` was invoked.",13,[[]]],[11,"after_owns","","Called after `owns` was invoked.",13,[[]]],[11,"by_ref","","Creates a \\\"by reference\\\" adaptor for this instance of…",13,[[]]]],"p":[[3,"RawRegion"],[3,"RawSharedRegion"],[3,"RawIntrusiveRegion"],[3,"Region"],[3,"SharedRegion"],[3,"IntrusiveRegion"],[3,"Counter"],[3,"AtomicCounter"],[4,"AllocInitFilter"],[4,"ReallocPlacementFilter"],[4,"ResultFilter"],[3,"FilteredCounter"],[3,"FilteredAtomicCounter"],[8,"CallbackRef"],[3,"Chunk"],[3,"Proxy"],[8,"AllocateAll"],[8,"ReallocateInPlace"],[8,"Owns"],[3,"Null"]]}\
}');
addSearchOptions(searchIndex);initSearch(searchIndex);