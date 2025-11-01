use tree_sitter::Tree;

// traverses the given tree and captures all nodes of the specified kind
pub fn traverse_and_capture<'a>(tree: &'a Tree, kind: &'a str) -> Vec<tree_sitter::Node<'a>> {
    let root = tree.root_node();
    traverse_and_capture_from_node(root, kind)
}

// traverses from a given node and captures all nodes of the specified kind
pub fn traverse_and_capture_from_node<'a>(
    node: tree_sitter::Node<'a>,
    kind: &'a str,
) -> Vec<tree_sitter::Node<'a>> {
    let mut captured_nodes = Vec::new();

    // stack for depth-first traversal to avoid lifetime issues
    let mut stack = vec![node];

    while let Some(node) = stack.pop() {
        if node.kind() == kind {
            captured_nodes.push(node);
        }
        let mut child_cursor = node.walk();
        if child_cursor.goto_first_child() {
            loop {
                stack.push(child_cursor.node());
                if !child_cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }

    captured_nodes
}

pub fn print_node(node: &tree_sitter::Node, text: &str) {
    println!(
        "Node: {} [{}-{}] \n text: '{}' \n\n",
        node.kind(),
        node.start_byte(),
        node.end_byte(),
        &text[node.start_byte()..node.end_byte()]
    );
}

// prints the tree structure starting from the given node
pub fn print_nodes(node: &tree_sitter::Node, depth: usize, text: &str, verbose: bool) {
    if verbose {
        println!(
            "{}Node: {} [{}-{}] \n text: '{}' \n\n",
            "  ".repeat(depth),
            node.kind(),
            node.start_byte(),
            node.end_byte(),
            &text[node.start_byte()..node.end_byte()]
        );
    } else {
        println!(
            "{}Node: {} [{}-{}]",
            "  ".repeat(depth),
            node.kind(),
            node.start_byte(),
            node.end_byte()
        );
    }
    let mut child_cursor = node.walk();
    if child_cursor.goto_first_child() {
        loop {
            print_nodes(&child_cursor.node(), depth + 1, text, verbose);
            if !child_cursor.goto_next_sibling() {
                break;
            }
        }
    }
}
