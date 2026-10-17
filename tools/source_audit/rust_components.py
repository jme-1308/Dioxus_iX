"""Read raw Rust component declarations; never expand macros or evaluate cfg."""

from tree_sitter import Language, Parser
import tree_sitter_rust


def text(source, node):
    return source[node.start_byte:node.end_byte].decode("utf-8")


def location(node):
    """Byte ranges are half-open; displayed source lines are one-based."""
    # Tuple access also avoids a Point.row accessor crash seen on Python 3.14.
    return {
        "start_byte": node.start_byte,
        "end_byte": node.end_byte,
        "start_line": node.start_point[0] + 1,
        "end_line": node.end_point[0] + 1,
    }


def descendants(node):
    pending = [node]
    while pending:
        current = pending.pop()
        yield current
        pending.extend(reversed(current.named_children))


def attribute_name(source, node):
    attribute = next((c for c in node.named_children if c.type == "attribute"), None)
    if attribute is None or not attribute.named_children:
        return ""
    return text(source, attribute.named_children[0]).split("::")[-1].removeprefix("r#")


def attribute_identifiers(source, node):
    return {text(source, child) for child in descendants(node)
            if child.type in ("identifier", "type_identifier")}


def inventory_source(source, path):
    """Return declarations and explicit diagnostics for one UTF-8 source file."""
    source.decode("utf-8")
    parser = Parser(Language(tree_sitter_rust.language()))
    tree = parser.parse(source)
    components, diagnostics = [], []

    def diagnose(node, code, message, severity="warning"):
        diagnostics.append({"path": path, **location(node), "severity": severity,
                            "code": code, "message": message})

    for node in descendants(tree.root_node):
        if node.is_error or node.is_missing:
            diagnose(node, "parse_error", f"Unexpected or missing Rust syntax: {node.type}", "error")

    def parameters(node):
        result, attributes = [], []
        for child in node.named_children:
            if child.type == "attribute_item":
                attributes.append(child)
            elif child.type in ("line_comment", "block_comment"):
                continue
            elif child.type == "parameter":
                pattern = child.child_by_field_name("pattern")
                value_type = child.child_by_field_name("type")
                if pattern is None or value_type is None:
                    diagnose(child, "unsupported_parameter", "Parameter has no explicit pattern/type.")
                else:
                    props = [a for a in attributes if attribute_name(source, a) == "props"]
                    result.append({
                        "pattern": text(source, pattern),
                        "type": text(source, value_type),
                        "declaration": text(source, child),
                        "attributes": [text(source, a) for a in attributes],
                        # Preserve exact default expressions; do not evaluate Rust.
                        "props_attributes": [text(source, a) for a in props],
                        "range": location(child),
                        "type_range": location(value_type),
                    })
                attributes = []
            else:
                diagnose(child, "unsupported_parameter", f"Unsupported parameter syntax: {child.type}")
                attributes = []
        return result

    def visit(container, scopes, enclosing_attributes):
        attributes, docs = [], []
        inner = [text(source, child) for child in container.named_children
                 if child.type == "inner_attribute_item"]
        if inner:
            enclosing_attributes = enclosing_attributes + [{"scope": scopes, "attributes": inner}]
        for node in container.named_children:
            if node.type == "inner_attribute_item":
                continue
            if node.type == "attribute_item":
                attributes.append(node)
                continue
            if node.type in ("line_comment", "block_comment"):
                if node.child_by_field_name("outer") is not None:
                    docs.append(text(source, node).rstrip())
                continue
            raw_attributes = [text(source, a) for a in attributes]
            for attribute in attributes:
                if (attribute_name(source, attribute) == "cfg_attr"
                        and "component" in attribute_identifiers(source, attribute)):
                    diagnose(attribute, "conditional_component",
                             "cfg_attr may introduce a component; conditional attributes are not evaluated.")
            is_component = any(attribute_name(source, a) == "component" for a in attributes)
            if is_component:
                if node.type != "function_item":
                    diagnose(node, "unsupported_component", "Component attribute is not on a function item.")
                else:
                    name = node.child_by_field_name("name")
                    arguments = node.child_by_field_name("parameters")
                    visibility = next((c for c in node.named_children
                                       if c.type == "visibility_modifier"), None)
                    return_type = node.child_by_field_name("return_type")
                    components.append({
                        "path": path,
                        "name": text(source, name),
                        "scope": scopes,
                        "visibility": text(source, visibility) if visibility else "private",
                        "attributes": raw_attributes,
                        "enclosing_attributes": enclosing_attributes,
                        "documentation": docs,
                        "range": location(node),
                        "name_range": location(name),
                        "parameters_range": location(arguments),
                        "parameters": parameters(arguments),
                        "return_type": text(source, return_type) if return_type else None,
                    })
            if node.type == "struct_item" and any(
                attribute_name(source, a) == "derive" and "Props" in attribute_identifiers(source, a)
                for a in attributes
            ):
                diagnose(node, "explicit_props_struct",
                         "Explicit Props struct found; linking it to manually declared components is not implemented.")
            # Macro bodies are token trees, not expanded component declarations.
            if node.type not in ("macro_invocation", "macro_definition", "attribute_item"):
                name = node.child_by_field_name("name")
                scoped = node.type in ("mod_item", "function_item", "impl_item", "trait_item")
                child_scopes = scopes + [text(source, name) if name else node.type] if scoped else scopes
                inherited = enclosing_attributes
                if raw_attributes:
                    inherited = enclosing_attributes + [{"scope": child_scopes, "attributes": raw_attributes}]
                visit(node, child_scopes, inherited)
            attributes, docs = [], []

    visit(tree.root_node, [], [])
    return components, diagnostics
