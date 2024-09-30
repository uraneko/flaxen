use crate::components::{ComponentTree, Container, Term, Text};
use crate::space::{border::Border, padding::Padding, Area, Pos};

// NOTE: for the overlay case
// TODO: prepare_overlay for both term and container
// that generates only a buffer of the parts that need to be rendered in the terminal display

// no overlay case
impl Term {
    // overlay behavior
    // to overlay
    // components can cover each other (overlap)
    //
    // or not to overlay
    // wrapper for the overlay shifting methods
    pub fn shift_no_overlay(&self, top: i16, bottom: i16, right: i16, left: i16) -> i16 {
        if top > 0 && bottom < 0 && right < 0 && left > 0 {
            self.shift_top_bottom_left_right(top, right, bottom, left)
        } else if top > 0 && right < 0 && left > 0 {
            self.shift_top_right_left(top, right, left)
        } else if bottom < 0 && right < 0 && left > 0 {
            self.shift_bottom_right_left(top, right, left)
        } else if top > 0 && bottom < 0 && right < 0 {
            self.shift_top_right_bottom(top, right, bottom)
        } else if top > 0 && bottom < 0 && left > 0 {
            self.shift_top_bottom_left(top, bottom, left)
        } else if bottom < 0 && right < 0 {
            self.shift_right_bottom(right, bottom)
        } else if bottom < 0 && left > 0 {
            self.shift_bottom_left(bottom, left)
        } else if top > 0 && right < 0 {
            self.shift_top_right(top, right)
        } else if top > 0 && left > 0 {
            self.shift_top_left(top, left)
        } else {
            0
        }
    }

    fn shift_top_left(&self, top: i16, left: i16) -> i16 {
        let al = i16::abs(left);
        let at = i16::abs(top);

        match al > at {
            // move new component to the left by left value
            // after checking that the parent area still has space
            true => al,

            // move new component to the top by top value
            // after checking that the parent area still has space
            false => at,
        }
    }

    fn shift_top_right(&self, top: i16, right: i16) -> i16 {
        let at = i16::abs(top);
        let ar = i16::abs(right);

        // if top/bottom == right/left then shift by right/left
        // since the window usually has more width than height
        if at > ar {
            at
        } else {
            ar
        }
    }

    fn shift_bottom_left(&self, bottom: i16, left: i16) -> i16 {
        let ab = i16::abs(bottom);
        let al = i16::abs(left);

        if ab > al {
            ab
        } else {
            al
        }
    }

    fn shift_right_bottom(&self, right: i16, bottom: i16) -> i16 {
        let ab = i16::abs(bottom);
        let ar = i16::abs(right);

        if ab > ar {
            ab
        } else {
            ar
        }
    }

    fn shift_top_right_left(&self, top: i16, right: i16, left: i16) -> i16 {
        let at = i16::abs(top);
        let ar = i16::abs(right);
        let al = i16::abs(left);

        // if equal we shift to the right since we start from the left
        if at > ar && at > al {
            at
        } else if al > at && al > ar {
            al
        } else {
            ar
        }
    }

    fn shift_bottom_right_left(&self, bottom: i16, right: i16, left: i16) -> i16 {
        let ab = i16::abs(bottom);
        let ar = i16::abs(right);
        let al = i16::abs(left);

        // if equal we shift to the right since we start from the left
        if ab > ar && ab > al {
            ab
        } else if al > ab && al > ar {
            al
        } else {
            ar
        }
    }

    fn shift_top_bottom_left(&self, top: i16, bottom: i16, left: i16) -> i16 {
        let ab = i16::abs(bottom);
        let at = i16::abs(top);
        let al = i16::abs(left);

        if at > ab && at > al {
            at
        } else if ab > at && ab > al {
            ab
        } else {
            al
        }
    }

    fn shift_top_right_bottom(&self, top: i16, right: i16, bottom: i16) -> i16 {
        let ab = i16::abs(bottom);
        let at = i16::abs(top);
        let ar = i16::abs(right);

        if at > ab && at > ar {
            at
        } else if ab > at && ab > ar {
            ab
        } else {
            ar
        }
    }

    fn shift_top_bottom_left_right(&self, top: i16, right: i16, bottom: i16, left: i16) -> i16 {
        let at = i16::abs(top);
        let ab = i16::abs(bottom);
        let ar = i16::abs(right);
        let al = i16::abs(left);

        if at > al && at > ar && at > ab {
            at
        } else if ab > at && ab > ar && ab > al {
            ab
        } else if al > at && al > ab && al > ar {
            al
        } else {
            ar
        }
    }
}

impl Container {
    // overlay behavior
    // to overlay
    // components can cover each other (overlap), render prepare_overlay
    //
    // or not to overlay
    // wrapper for the overlay shifting methods
    pub fn shift_no_overlay(&self, top: i16, right: i16, bottom: i16, left: i16) -> i16 {
        if top > 0 && bottom < 0 && right < 0 && left > 0 {
            self.shift_top_bottom_left_right(top, right, bottom, left)
        } else if top > 0 && right < 0 && left > 0 {
            self.shift_top_right_left(top, right, left)
        } else if bottom < 0 && right < 0 && left > 0 {
            self.shift_bottom_right_left(top, right, left)
        } else if top > 0 && bottom < 0 && right < 0 {
            self.shift_top_right_bottom(top, right, bottom)
        } else if top > 0 && bottom < 0 && left > 0 {
            self.shift_top_bottom_left(top, bottom, left)
        } else if bottom < 0 && right < 0 {
            self.shift_right_bottom(right, bottom)
        } else if bottom < 0 && left > 0 {
            self.shift_bottom_left(bottom, left)
        } else if top > 0 && right < 0 {
            self.shift_top_right(top, right)
        } else if top > 0 && left > 0 {
            self.shift_top_left(top, left)
        } else {
            0
        }
    }

    fn shift_top_left(&self, top: i16, left: i16) -> i16 {
        let al = i16::abs(left);
        let at = i16::abs(top);

        match at > al {
            // move new component to the left by left value
            // after checking that the parent area still has space
            true => at,

            // move new component to the top by top value
            // after checking that the parent area still has space
            false => al,
        }
    }

    fn shift_top_right(&self, top: i16, right: i16) -> i16 {
        let at = i16::abs(top);
        let ar = i16::abs(right);

        // if top/bottom == right/left then shift by right/left
        // since the window usually has more width than height
        if at > ar {
            at
        } else {
            ar
        }
    }

    fn shift_bottom_left(&self, bottom: i16, left: i16) -> i16 {
        let ab = i16::abs(bottom);
        let al = i16::abs(left);

        if ab > al {
            ab
        } else {
            al
        }
    }

    fn shift_right_bottom(&self, right: i16, bottom: i16) -> i16 {
        let ab = i16::abs(bottom);
        let ar = i16::abs(right);

        if ab > ar {
            ab
        } else {
            ar
        }
    }

    fn shift_top_right_left(&self, top: i16, right: i16, left: i16) -> i16 {
        let at = i16::abs(top);
        let ar = i16::abs(right);
        let al = i16::abs(left);

        // if equal we shift to the right since we start from the left
        if at > ar && at > al {
            at
        } else if al > at && al > ar {
            al
        } else {
            ar
        }
    }

    fn shift_bottom_right_left(&self, bottom: i16, right: i16, left: i16) -> i16 {
        let ab = i16::abs(bottom);
        let ar = i16::abs(right);
        let al = i16::abs(left);

        // if equal we shift to the right since we start from the left
        if ab > ar && ab > al {
            ab
        } else if al > ab && al > ar {
            al
        } else {
            ar
        }
    }

    fn shift_top_bottom_left(&self, top: i16, bottom: i16, left: i16) -> i16 {
        let ab = i16::abs(bottom);
        let at = i16::abs(top);
        let al = i16::abs(left);

        if at > ab && at > al {
            at
        } else if ab > at && ab > al {
            ab
        } else {
            al
        }
    }

    fn shift_top_right_bottom(&self, top: i16, right: i16, bottom: i16) -> i16 {
        let ab = i16::abs(bottom);
        let at = i16::abs(top);
        let ar = i16::abs(right);

        if at > ab && at > ar {
            at
        } else if ab > at && ab > ar {
            ab
        } else {
            ar
        }
    }

    fn shift_top_bottom_left_right(&self, top: i16, right: i16, bottom: i16, left: i16) -> i16 {
        let at = i16::abs(top);
        let ab = i16::abs(bottom);
        let ar = i16::abs(right);
        let al = i16::abs(left);

        if at > al && at > ar && at > ab {
            at
        } else if ab > at && ab > ar && ab > al {
            ab
        } else if al > at && al > ab && al > ar {
            al
        } else {
            ar
        }
    }
}
