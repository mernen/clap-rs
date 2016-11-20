// Std
use std::fmt::{Display, Formatter, Result};
use std::rc::Rc;
use std::result::Result as StdResult;

// Third Party
use vec_map::VecMap;

// Internal
use args::{ArgSettings, ArgKind, AnyArg, Base, Switched, Valued, Arg, DispOrder};

#[allow(missing_debug_implementations)]
#[doc(hidden)]
#[derive(Default, Clone)]
pub struct OptBuilder<'n, 'e>
    where 'n: 'e
{
    pub b: Base<'n, 'e>,
    pub s: Switched<'e>,
    pub v: Valued<'n, 'e>,
}

impl<'n, 'e> OptBuilder<'n, 'e> {
    pub fn new(name: &'n str) -> Self { OptBuilder { b: Base::new(name), ..Default::default() } }

    pub fn from_arg(a: &Arg<'n, 'e>, reqs: &mut Vec<&'e str>) -> Self {
        // No need to check for .index() as that is handled above
        let ob = OptBuilder {
            b: Base::from(a),
            s: Switched::from(a),
            v: Valued::from(a),
        };
        // If the arg is required, add all it's requirements to master required list
        if a.is_set(ArgSettings::Required) {
            if let Some(ref areqs) = a.requires {
                reqs.extend(areqs);
            }
        }
        ob
    }
}

impl<'n, 'e> Display for OptBuilder<'n, 'e> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        debugln!("fn=fmt");
        // Write the name such --long or -l
        if let Some(l) = self.s.long {
            try!(write!(f, "--{} ", l));
        } else {
            try!(write!(f, "-{} ", self.s.short.unwrap()));
        }

        // Write the values such as <name1> <name2>
        if let Some(ref vec) = self.v.val_names {
            let mut it = vec.iter().peekable();
            while let Some((_, val)) = it.next() {
                try!(write!(f, "<{}>", val));
                if it.peek().is_some() {
                    try!(write!(f, " "));
                }
            }
            let num = vec.len();
            if self.is_set(ArgSettings::Multiple) && num == 1 {
                try!(write!(f, "..."));
            }
        } else if let Some(num) = self.v.num_vals {
            let mut it = (0..num).peekable();
            while let Some(_) = it.next() {
                try!(write!(f, "<{}>", self.b.name));
                if it.peek().is_some() {
                    try!(write!(f, " "));
                }
            }
        } else {
            try!(write!(f,
                        "<{}>{}",
                        self.b.name,
                        if self.is_set(ArgSettings::Multiple) {
                            "..."
                        } else {
                            ""
                        }));
        }

        Ok(())
    }
}

impl<'n, 'e> AnyArg<'n, 'e> for OptBuilder<'n, 'e> {
    fn name(&self) -> &'n str { self.b.name }
    fn kind(&self) -> ArgKind { ArgKind::Opt }
    fn overrides(&self) -> Option<&[&'e str]> { self.b.overrides.as_ref().map(|o| &o[..]) }
    fn requires(&self) -> Option<&[&'e str]> { self.b.requires.as_ref().map(|o| &o[..]) }
    fn blacklist(&self) -> Option<&[&'e str]> { self.b.blacklist.as_ref().map(|o| &o[..]) }
    fn required_unless(&self) -> Option<&[&'e str]> { self.b.r_unless.as_ref().map(|o| &o[..]) }
    fn val_names(&self) -> Option<&VecMap<&'e str>> { self.v.val_names.as_ref() }
    fn is_set(&self, s: ArgSettings) -> bool { self.b.settings.is_set(s) }
    fn has_switch(&self) -> bool { true }
    fn set(&mut self, s: ArgSettings) { self.b.settings.set(s) }
    fn max_vals(&self) -> Option<u64> { self.v.max_vals }
    fn num_vals(&self) -> Option<u64> { self.v.num_vals }
    fn possible_vals(&self) -> Option<&[&'e str]> { self.v.possible_vals.as_ref().map(|o| &o[..]) }
    fn validator(&self) -> Option<&Rc<Fn(String) -> StdResult<(), String>>> {
        self.v.validator.as_ref()
    }
    fn min_vals(&self) -> Option<u64> { self.v.min_vals }
    fn short(&self) -> Option<char> { self.s.short }
    fn long(&self) -> Option<&'e str> { self.s.long }
    fn val_delim(&self) -> Option<char> { self.v.val_delim }
    fn takes_value(&self) -> bool { true }
    fn help(&self) -> Option<&'e str> { self.b.help }
    fn default_val(&self) -> Option<&'n str> { self.v.default_val }
    fn longest_filter(&self) -> bool { true }
    fn aliases(&self) -> Option<Vec<&'e str>> {
        if let Some(ref aliases) = self.s.aliases {
            let vis_aliases: Vec<_> = aliases.iter()
                .filter_map(|&(n, v)| if v { Some(n) } else { None })
                .collect();
            if vis_aliases.is_empty() {
                None
            } else {
                Some(vis_aliases)
            }
        } else {
            None
        }
    }
}

impl<'n, 'e> DispOrder for OptBuilder<'n, 'e> {
    fn disp_ord(&self) -> usize { self.s.disp_ord }
}

#[cfg(test)]
mod test {
    use args::settings::ArgSettings;
    use super::OptBuilder;
    use vec_map::VecMap;

    #[test]
    fn optbuilder_display1() {
        let mut o = OptBuilder::new("opt");
        o.long = Some("option");
        o.settings.set(ArgSettings::Multiple);

        assert_eq!(&*format!("{}", o), "--option <opt>...");
    }

    #[test]
    fn optbuilder_display2() {
        let mut v_names = VecMap::new();
        v_names.insert(0, "file");
        v_names.insert(1, "name");

        let mut o2 = OptBuilder::new("opt");
        o2.short = Some('o');
        o2.val_names = Some(v_names);

        assert_eq!(&*format!("{}", o2), "-o <file> <name>");
    }

    #[test]
    fn optbuilder_display3() {
        let mut v_names = VecMap::new();
        v_names.insert(0, "file");
        v_names.insert(1, "name");

        let mut o2 = OptBuilder::new("opt");
        o2.short = Some('o');
        o2.val_names = Some(v_names);
        o2.settings.set(ArgSettings::Multiple);

        assert_eq!(&*format!("{}", o2), "-o <file> <name>");
    }

    #[test]
    fn optbuilder_display_single_alias() {
        let mut o = OptBuilder::new("opt");
        o.long = Some("option");
        o.aliases = Some(vec![("als", true)]);

        assert_eq!(&*format!("{}", o), "--option <opt>");
    }

    #[test]
    fn optbuilder_display_multiple_aliases() {
        let mut o = OptBuilder::new("opt");
        o.long = Some("option");
        o.aliases =
            Some(vec![("als_not_visible", false), ("als2", true), ("als3", true), ("als4", true)]);
        assert_eq!(&*format!("{}", o), "--option <opt>");
    }
}
