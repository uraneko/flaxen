use crate::components::{ComponentTree, Container, SpaceError, Term, Text};

#[derive(Debug, Clone, Default)]
pub enum Layout {
    #[default]
    /// no particular layout rules are applied on the children
    /// every child will follow its area and position
    Canvas,
    /// children are displayed in a flex style
    /// for more customization add a "flex" map property to this component
    /// with the needed properties
    Flex { direction: char },
    /// children are displayed in a grid style
    /// for more customization add a "grid" map property to this component
    /// with the needed properties
    Grid { cols: u8, rows: u8 },
}

// TODO: if flex/grid then apply rules on every comp_push()/comp() methods
// both Flex and Grid would have to use the attributes and properties feature

impl Term {
    pub(crate) fn is_valid_nonedit_id(&self, id: &[u8; 3]) -> bool {
        id[2] % 2 != 0 && self.has_container(&[id[0], id[1]]) && !self.has_nonedit(&id)
    }

    pub(crate) fn is_valid_input_id(&self, id: &[u8; 3]) -> bool {
        id[2] % 2 == 0 && self.has_container(&[id[0], id[1]]) && !self.has_input(&id)
    }

    pub(crate) fn is_valid_container_id(&self, id: &[u8; 2]) -> bool {
        !self.has_container(&id)
    }
}

impl Container {
    pub(crate) fn area_out_of_bounds(&self, wh: &[u16; 2]) -> bool {
        let [w, h] = *wh;
        if self.w * self.h < w * h || w > self.w || h > self.h {
            return true;
        }

        false
    }

    // flex and grid dont respect origins
    pub(crate) fn origin_out_of_bounds(&self, xy: &[u16; 2], wh: &[u16; 2]) -> bool {
        let [x0, y0] = *xy;
        let [w, h] = *wh;
        x0 > self.w || y0 > self.h || x0 + w > self.w || y0 + h > self.h
    }

    // TODO: make width/height augmented by paddings/border or both

    fn input_from_meta() {}
    fn input_canvas() {}
    fn input_flex() {}
    fn input_grid() {}

    // caculate new child x0 y0 to fit flex layout of this parent
    fn layout_flex(&self, text: &mut Text) {
        self.items.iter().map(|t| t);
    }

    // calculate new child x0 y0 to fit grid layout of this parent
    fn layout_grid(&self, text: &mut Text) {}

    fn input_space_validation(&self, mut text: Text) -> Result<Text, SpaceError> {
        if self.area_out_of_bounds(&[text.w, text.h]) {
            return Err(SpaceError::AreaOutOfBounds);
        } else if self.origin_out_of_bounds(&[text.w, text.h], &[text.x0, text.y0]) {
            return Err(SpaceError::OriginOutOfBounds);
        }

        Ok(text)
    }
}

// checks for adding a component to its parent
// 1/ id check
// 2/ layout checks
//      2.1/ check parent layout
//      2.2/ if layout is flex then ignore x0 and y0 and apply flex rules
//          2.2.1/ before applying flex rules check overlay then check area bounds
//          2.2.2/ if overlay is off and area bounds are not respected abort with error else apply
//            rules and accept
//          2.2.3/ if overlay is on then as long as component area is not bigger than parent area
//            it will be accepted
//      2.3/ if layout is grid then ignore x0 and y0 and apply grid rules
//          2.3.1/ before applying grid rules check overlay then check area bounds
//          2.3.2/ if overlay is off and area bounds are not respected abort with error else appply
//            rules and accept
//          2.3.3/ if overlay is on then as long as component area is not bigger than parent area
//            it will be accepted
//      2.4/ if layout is canvas then don't apply any additional rules
//          2.4.1/ check overlay then check area bounds
//          2.4.2/ if overlay is off and area bounds are not respected abort with error else accept
//          2.4.3/ if overlay is on then as long as component area is not bigger than parent area
//            it will be accepted
