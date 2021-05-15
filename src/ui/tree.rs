use std::collections::HashMap;

use crate::ui::*;

pub enum UiTree<T> {
    Leaf(Vec<T>),
    Children(Vec<UiTree<T>>)
}
impl<'a, ItemType> IntoIterator for &'a UiTree<ItemType> {
    type Item = &'a ItemType;
    type IntoIter = std::vec::IntoIter<&'a ItemType>;

    fn into_iter(self) -> Self::IntoIter {
        fn append<'a, T>(tree: &'a UiTree<T>, v: &mut Vec<&'a T>) {
            match tree {
                &UiTree::Leaf(ref items) => {
                    v.extend(items.iter());
                },
                &UiTree::Children(ref children) => {
                    for child in children {
                        append(child, v);
                    }
                }
            }
        }

        let mut result = vec![];
        append(self, &mut result);
        result.into_iter()
    }
}
impl<'a, ItemType> IntoIterator for &'a mut UiTree<ItemType> {
    type Item = &'a mut ItemType;
    type IntoIter = std::vec::IntoIter<&'a mut ItemType>;

    fn into_iter(self) -> Self::IntoIter {
        fn append<'a, T>(tree: &'a mut UiTree<T>, v: &mut Vec<&'a mut T>) {
            match tree {
                &mut UiTree::Leaf(ref mut items) => {
                    v.extend(items.iter_mut());
                },
                &mut UiTree::Children(ref mut children) => {
                    for child in children {
                        append(child, v);
                    }
                }
            }
        }

        let mut result = vec![];
        append(self, &mut result);
        result.into_iter()
    }
}

// descriptor to set ui and create changeable text objects
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiComponent {
    InfoTag(InfoTag),
    InfoText(String),
    List,
    CornerInfoBox,
    InfoBar,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum UiNodeType {
    LeftCornerBox,
    TopBar,
}

impl UiComponent {
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiNode {
    node_type: UiNodeType,
    children: Vec<UiComponent>,
    entity: Option<Entity>,
    redraw: bool,
}

pub struct UiConductor {
    nodes: HashMap<UiNodeType, UiNode>,
}

impl UiConductor {
    pub fn merge(&mut self, new: UiNode) {
        if let Some(mut node) = self.nodes.get_mut(&new.node_type) {
            node.children = new.children.clone();
            node.redraw = true;
        }
    }
}

pub fn ui_conductor_render_system(
    ui_conductor: Res<UiConductor>,
) {
    for ui_node_type in UiNodeType::iter() {
        if let Some(node) = ui_conductor.nodes.get(&ui_node_type) {
            if !node.redraw {
                break;
            }

        }
    }
}
