/// Container and Text objects border
#[derive(Debug, Default, Clone, Copy)]
pub enum Border {
    /// no border
    #[default]
    None,

    /// same character border
    Uniform(char),

    /// border with different chars for the corners and the sides
    Polyform {
        /// top right corner border char
        trcorner: char,
        /// top left corner border char
        tlcorner: char,
        /// bottom left corner border char
        blcorner: char,
        /// bottom right corner border char
        brcorner: char,
        /// right/left sides border char
        rl: char,
        /// top/bottom sides border char
        tb: char,
    },

    Manual {
        /// top left corner
        tlcorner: char,
        /// top right corner
        trcorner: char,
        /// bottom right corner
        brcorner: char,
        /// bottom left corner
        blcorner: char,

        // top side
        t0: &'static str,
        tp: char,
        t1: &'static str,

        // right side
        r0: &'static str,
        rp: char,
        r1: &'static str,

        //
        l0: &'static str,
        lp: char,
        l1: &'static str,

        b0: &'static str,
        bp: char,
        b1: &'static str,
    },
}

// FIXME: since the Manual variant takes 'static lifetimed strs
// it can not be created nor its method manual() used with strs that are static like those gotten
// from String::as_str
// keep like this or make it take String
// TODO: find out why i put copy trait on Border enum
impl Border {
    /// creates a new Border with the None variant
    pub fn none() -> Self {
        Self::None
    }

    /// creates a new Border with the Uniform variant
    pub fn uniform(value: char) -> Self {
        Self::Uniform(value)
    }

    /// creates a new Border with the Polyform variant
    pub fn polyform(
        tlcorner: char,
        trcorner: char,
        brcorner: char,
        blcorner: char,
        rl: char,
        tb: char,
    ) -> Self {
        Self::Polyform {
            trcorner,
            tlcorner,
            brcorner,
            blcorner,
            rl,
            tb,
        }
    }

    /// creates a new Border with the Manual variant
    pub fn manual(
        tlcorner: char,
        trcorner: char,
        brcorner: char,
        blcorner: char,
        t0: &'static str,
        tp: char,
        t1: &'static str,
        r0: &'static str,
        rp: char,
        r1: &'static str,
        l0: &'static str,
        lp: char,
        l1: &'static str,
        b0: &'static str,
        bp: char,
        b1: &'static str,
    ) -> Self {
        Self::Manual {
            trcorner,
            tlcorner,
            brcorner,
            blcorner,
            t0,
            tp,
            t1,
            r0,
            rp,
            r1,
            l0,
            lp,
            l1,
            b0,
            bp,
            b1,
        }
    }

    pub fn mono(self, mono: char) -> Self {
        if let Self::Uniform(ch) = self {
            return Self::Uniform(mono);
        }

        self
    }

    pub fn trcorner(self, trcorner: char) -> Self {
        if let Self::Polyform {
            tlcorner,
            blcorner,
            brcorner,
            rl,
            tb,
            ..
        } = self
        {
            return Self::Polyform {
                trcorner,
                tlcorner,
                blcorner,
                brcorner,
                rl,
                tb,
            };
        } else if let Self::Manual {
            tlcorner,
            blcorner,
            brcorner,

            t0,
            tp,
            t1,

            r0,
            rp,
            r1,

            l0,
            lp,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn tlcorner(self, tlcorner: char) -> Self {
        if let Self::Polyform {
            trcorner,
            blcorner,
            brcorner,
            rl,
            tb,
            ..
        } = self
        {
            return Self::Polyform {
                trcorner,
                tlcorner,
                blcorner,
                brcorner,
                rl,
                tb,
            };
        } else if let Self::Manual {
            blcorner,
            trcorner,
            brcorner,

            t0,
            tp,
            t1,

            r0,
            rp,
            r1,

            l0,
            lp,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn brcorner(self, brcorner: char) -> Self {
        if let Self::Polyform {
            tlcorner,
            blcorner,
            trcorner,
            rl,
            tb,
            ..
        } = self
        {
            return Self::Polyform {
                trcorner,
                tlcorner,
                blcorner,
                brcorner,
                rl,
                tb,
            };
        } else if let Self::Manual {
            tlcorner,
            trcorner,
            blcorner,

            t0,
            tp,
            t1,

            r0,
            rp,
            r1,

            l0,
            lp,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn blcorner(self, blcorner: char) -> Self {
        if let Self::Polyform {
            tlcorner,
            brcorner,
            trcorner,
            rl,
            tb,
            ..
        } = self
        {
            return Self::Polyform {
                trcorner,
                tlcorner,
                blcorner,
                brcorner,
                rl,
                tb,
            };
        } else if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,

            t0,
            tp,
            t1,

            r0,
            rp,
            r1,

            l0,
            lp,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn t0(self, t0: &'static str) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            tp,
            t1,

            r0,
            rp,
            r1,

            l0,
            lp,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn t1(self, t1: &'static str) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            tp,
            t0,

            r0,
            rp,
            r1,

            l0,
            lp,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn b0(self, b0: &'static str) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            t0,
            tp,
            t1,

            r0,
            rp,
            r1,

            l0,
            lp,
            l1,

            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn b1(self, b1: &'static str) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            t0,
            tp,
            t1,

            r0,
            rp,
            r1,

            l0,
            lp,
            l1,

            b0,
            bp,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn r0(self, r0: &'static str) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            t0,
            tp,
            t1,

            rp,
            r1,

            l0,
            lp,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn r1(self, r1: &'static str) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            t0,
            tp,
            t1,

            r0,
            rp,

            l0,
            lp,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn l0(self, l0: &'static str) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            t0,
            tp,
            t1,

            r0,
            rp,
            r1,

            lp,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn l1(self, l1: &'static str) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            t0,
            tp,
            t1,
            r0,
            rp,
            r1,

            l0,
            lp,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn tp(self, tp: char) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            t0,
            t1,

            r0,
            rp,
            r1,

            l0,
            lp,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn bp(self, bp: char) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            t0,
            tp,
            t1,

            r0,
            rp,
            r1,

            l0,
            lp,
            l1,

            b0,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn rp(self, rp: char) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            t0,
            tp,
            t1,

            r0,
            r1,

            l0,
            lp,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn lp(self, lp: char) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            t0,
            tp,
            t1,

            r0,
            rp,
            r1,

            l0,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }
}
