#[macro_use]
mod macros;
mod bash;
mod fish;
mod zsh;
mod powershell;
mod shell;

// Std
use std::io::Write;

// Internal
use app::parser::Parser;
use self::bash::BashGen;
use self::fish::FishGen;
use self::zsh::ZshGen;
use self::powershell::PowerShellGen;
pub use self::shell::Shell;

pub struct ComplGen<'a, 'b>
    where 'a: 'b
{
    p: &'b Parser<'a, 'b>,
}

impl<'a, 'b> ComplGen<'a, 'b> {
    pub fn new(p: &'b Parser<'a, 'b>) -> Self { ComplGen { p: p } }

    pub fn generate<W: Write>(&self, for_shell: Shell, buf: &mut W) {
        match for_shell {
            Shell::Bash => BashGen::new(self.p).generate_to(buf),
            Shell::Fish => FishGen::new(self.p).generate_to(buf),
            Shell::Zsh => ZshGen::new(self.p).generate_to(buf),
            Shell::PowerShell => PowerShellGen::new(self.p).generate_to(buf),
        }
    }
}

// Gets all subcommands including child subcommands in the form of 'name' where the name
// is a single word (i.e. "install")  of the path to said subcommand (i.e.
// "rustup toolchain install")
//
// Also note, aliases are treated as their own subcommands but duplicates of whatever they're
// aliasing.
pub fn all_subcommand_names(p: &Parser) -> Vec<String> {
    let mut subcmds: Vec<_> = subcommands_of(p).iter().map(|&(ref n, _)| n.clone()).collect();
    for sc_v in p.subcommands.iter().map(|s| all_subcommand_names(&s.p)) {
        subcmds.extend(sc_v);
    }
    subcmds.sort();
    subcmds.dedup();
    subcmds
}

// Gets all subcommands including child subcommands in the form of ('name', 'bin_name') where the name
// is a single word (i.e. "install") of the path and full bin_name of said subcommand (i.e.
// "rustup toolchain install")
//
// Also note, aliases are treated as their own subcommands but duplicates of whatever they're
// aliasing.
pub fn all_subcommands(p: &Parser) -> Vec<(String, String)> {
    let mut subcmds: Vec<_> = subcommands_of(p);
    for sc_v in p.subcommands.iter().map(|s| all_subcommands(&s.p)) {
        subcmds.extend(sc_v);
    }
    subcmds
}

// Gets all subcommands exlcuding child subcommands in the form of (name, bin_name) where the name
// is a single word (i.e. "install") and the bin_name is a space deliniated list of the path to said
// subcommand (i.e. "rustup toolchain install")
//
// Also note, aliases are treated as their own subcommands but duplicates of whatever they're
// aliasing.
pub fn subcommands_of(p: &Parser) -> Vec<(String, String)> {
    debugln!("fn=subcommands_of;name={};bin_name={}",
             p.meta.name,
             p.meta.bin_name.as_ref().unwrap());
    let mut subcmds = vec![];

    debug!("Has subcommands...");
    if !p.has_subcommands() {
        sdebugln!("No");
        let mut ret = vec![(p.meta.name.clone(), p.meta.bin_name.as_ref().unwrap().clone())];
        debugln!("Looking for aliases...");
        if let Some(ref aliases) = p.meta.aliases {
            for &(n, _) in aliases {
                debugln!("Found alias...{}", n);
                let mut als_bin_name: Vec<_> =
                    p.meta.bin_name.as_ref().unwrap().split(' ').collect();
                als_bin_name.push(n);
                let old = als_bin_name.len() - 2;
                als_bin_name.swap_remove(old);
                ret.push((n.to_owned(), als_bin_name.join(" ")));
            }
        }
        return ret;
    }
    sdebugln!("Yes");
    for sc in &p.subcommands {
        debugln!("iter;name={};bin_name={}",
                 sc.p.meta.name,
                 sc.p.meta.bin_name.as_ref().unwrap());

        debugln!("Looking for aliases...");
        if let Some(ref aliases) = sc.p.meta.aliases {
            for &(n, _) in aliases {
                debugln!("Found alias...{}", n);
                let mut als_bin_name: Vec<_> =
                    p.meta.bin_name.as_ref().unwrap().split(' ').collect();
                als_bin_name.push(n);
                let old = als_bin_name.len() - 2;
                als_bin_name.swap_remove(old);
                subcmds.push((n.to_owned(), als_bin_name.join(" ")));
            }
        }
        subcmds.push((sc.p.meta.name.clone(), sc.p.meta.bin_name.as_ref().unwrap().clone()));
    }
    subcmds
}

pub fn get_all_subcommand_paths(p: &Parser, first: bool) -> Vec<String> {
    let mut subcmds = vec![];
    if !p.has_subcommands() {
        if !first {
            let name = &*p.meta.name;
            let path = p.meta.bin_name.as_ref().unwrap().clone().replace(" ", "_");
            let mut ret = vec![path.clone()];
            if let Some(ref aliases) = p.meta.aliases {
                for &(n, _) in aliases {
                    ret.push(path.replace(name, n));
                }
            }
            return ret;
        }
        return vec![];
    }
    for sc in &p.subcommands {
        let name = &*sc.p.meta.name;
        let path = sc.p.meta.bin_name.as_ref().unwrap().clone().replace(" ", "_");
        subcmds.push(path.clone());
        if let Some(ref aliases) = sc.p.meta.aliases {
            for &(n, _) in aliases {
                subcmds.push(path.replace(name, n));
            }
        }
    }
    for sc_v in p.subcommands.iter().map(|s| get_all_subcommand_paths(&s.p, false)) {
        subcmds.extend(sc_v);
    }
    subcmds
}
