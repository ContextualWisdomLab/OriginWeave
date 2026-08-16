use originweave_core::mcp::{McpCacheScope, mcp_tools_list_page, supported_mcp_tools};

#[test]
fn mcp_tools_list_page_is_complete_private_and_immediately_stale() {
    let page = mcp_tools_list_page();

    assert_eq!(page.tools(), supported_mcp_tools());
    assert_eq!(page.ttl_ms(), 0);
    assert_eq!(page.cache_scope(), McpCacheScope::Private);
    assert_eq!(page.next_cursor(), None);
}
