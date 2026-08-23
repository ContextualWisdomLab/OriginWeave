use originweave_policy::EnterpriseApprovalRequest;

#[test]
fn approval_accounting_state_is_not_cloneable() {
    trait AmbiguousIfClone<A> {
        fn marker() {}
    }

    impl<T: ?Sized> AmbiguousIfClone<()> for T {}

    struct CloneImplemented;
    impl<T: ?Sized + Clone> AmbiguousIfClone<CloneImplemented> for T {}

    let _ = <EnterpriseApprovalRequest as AmbiguousIfClone<_>>::marker;
}
