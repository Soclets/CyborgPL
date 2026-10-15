//! The power ledger (`cca/PLUGINS.md` §4, step 2; DECISIONS 2026-10-07).
//!
//! The codextra reports every use of a power; the base checks each one,
//! in this order, with its own message:
//!
//! 1. **granted**: by the codextra to every limb, or by a core the limb is
//!    built with;
//! 2. **listed**: in the limb's own `powers`, the program's limb included;
//! 3. **not denied**: by the program's `deny`, with `.except[…]` and
//!    `.for[…]`.
//!
//! A lone file has every core and every power, and nothing is checked.
//! The base does not trust the codextra to have checked: it checks what it
//! is told was used.

/// A power: a path of words, narrowed with `.`, and the exact things it
/// is about, named in `[ ]`: `c_calls`, `c_calls.functions`,
/// `c_calls.functions["write"]`, `vouch.c`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Power {
    pub path: Vec<String>,
    /// Empty: every thing.
    pub things: Vec<String>,
}

impl Power {
    pub fn new(path: &str) -> Power {
        Power { path: path.split('.').map(str::to_string).collect(), things: Vec::new() }
    }

    pub fn with(mut self, things: &[&str]) -> Power {
        self.things = things.iter().map(|t| t.to_string()).collect();
        self
    }

    /// `c_calls.functions`, as written after `cca::Power.`.
    pub fn spelled(&self) -> String {
        let path = self.path.join(".");
        if self.things.is_empty() {
            path
        } else {
            format!("{}[{}]", path, self.things.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join(", "))
        }
    }

    /// Whether this power, listed or denied, covers a use: its path is the
    /// use's or a prefix of it, and if it names things, the use's thing is
    /// one of them.
    pub fn covers(&self, used: &Use) -> bool {
        self.path.len() <= used.path.len()
            && self.path.iter().zip(&used.path).all(|(a, b)| a == b)
            && (self.things.is_empty() || used.thing.as_ref().is_some_and(|t| self.things.contains(t)))
    }
}

/// One use of a power, as the codextra reports it: which limb, the most
/// specific path, the thing (a C function's name), and where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Use {
    /// The limb's name, as its manifest gives it.
    pub limb: String,
    pub path: Vec<String>,
    pub thing: Option<String>,
    /// What to say in a message: "calls C (`write`)".
    pub doing: String,
    /// Where, for the message; the codextra's own span type stays its own.
    pub at: u32,
}

/// Who may use what: a grant names a power and either every limb built
/// with its giver, or the limbs it favours by registry ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grant {
    pub power: Power,
    /// Who grants it: the codextra, or a core, by name for messages.
    pub by: String,
    /// `None`: every limb built with the giver; `Some`: these registry IDs.
    pub to: Option<Vec<String>>,
}

/// An entry of the program's `deny`: a power, the things it leaves alone
/// (`.except[…]`), and the limbs it is about (`.for[…]`; empty, every limb).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deny {
    pub power: Power,
    pub except: Vec<String>,
    pub for_limbs: Vec<String>,
}

impl Deny {
    pub fn covers(&self, used: &Use) -> bool {
        Power { path: self.power.path.clone(), things: self.power.things.clone() }.covers(used)
            && (self.for_limbs.is_empty() || self.for_limbs.contains(&used.limb))
            && !used.thing.as_ref().is_some_and(|t| self.except.contains(t))
    }
}

/// One limb, as the ledger needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Limb {
    pub name: String,
    /// Its registry ID, when it has one: favourites are named by it.
    pub id: Option<String>,
    /// The powers it lists.
    pub lists: Vec<Power>,
    /// What it is built with: the grants that reach it.
    pub grants: Vec<Grant>,
}

/// Why a use is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    NotGranted { used: Use },
    NotListed { used: Use },
    Denied { used: Use, entry: Deny },
}

impl Refusal {
    pub fn sentence(&self) -> String {
        let top = |u: &Use| u.path[0].clone();
        match self {
            Refusal::NotGranted { used } => format!("`{}` {}, and nothing it is built with grants `{}`", used.limb, used.doing, used.path.join(".")),
            Refusal::NotListed { used } => format!("`{}` {}, and its CCA.cplr does not list `{}`: add `cca::Power.{}` to its `powers`", used.limb, used.doing, top(used), top(used)),
            Refusal::Denied { used, entry } => format!("`{}` {}, and the program denies `{}` (its CCA.cplr, `deny`)", used.limb, used.doing, entry.power.spelled()),
        }
    }
}

/// Check one use against its limb and the program's `deny`, in the order
/// the module says.
pub fn check(used: &Use, limb: &Limb, deny: &[Deny]) -> Result<(), Refusal> {
    let granted = limb.grants.iter().any(|g| {
        g.power.covers(used) && g.to.as_ref().map_or(true, |ids| limb.id.as_ref().is_some_and(|id| ids.contains(id)))
    });
    if !granted {
        return Err(Refusal::NotGranted { used: used.clone() });
    }
    if !limb.lists.iter().any(|p| p.covers(used)) {
        return Err(Refusal::NotListed { used: used.clone() });
    }
    if let Some(entry) = deny.iter().find(|d| d.covers(used)) {
        return Err(Refusal::Denied { used: used.clone(), entry: entry.clone() });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c_call(limb: &str, name: &str) -> Use {
        Use { limb: limb.into(), path: vec!["c_calls".into(), "functions".into()], thing: Some(name.into()), doing: format!("calls C (`{}`)", name), at: 0 }
    }

    fn signatar() -> Vec<Grant> {
        ["inline_assembly", "c_calls", "raw_memory"].iter().map(|p| Grant { power: Power::new(p), by: "Signatar".into(), to: None }).collect()
    }

    fn limb(name: &str, lists: &[Power]) -> Limb {
        Limb { name: name.into(), id: None, lists: lists.to_vec(), grants: signatar() }
    }

    #[test]
    fn a_listed_granted_power_is_used() {
        assert_eq!(check(&c_call("shop", "write"), &limb("shop", &[Power::new("c_calls")]), &[]), Ok(()));
        // A narrower listing covers a narrower use.
        assert_eq!(check(&c_call("shop", "write"), &limb("shop", &[Power::new("c_calls.functions").with(&["write"])]), &[]), Ok(()));
    }

    #[test]
    fn unlisted_is_refused_naming_the_power() {
        let r = check(&c_call("fastjson", "write"), &limb("fastjson", &[]), &[]).unwrap_err();
        assert_eq!(r.sentence(), "`fastjson` calls C (`write`), and its CCA.cplr does not list `c_calls`: add `cca::Power.c_calls` to its `powers`");
        // Listing other things is not listing this one.
        assert!(check(&c_call("fastjson", "write"), &limb("fastjson", &[Power::new("c_calls").with(&["sqrt"])]), &[]).is_err());
    }

    #[test]
    fn ungranted_comes_first() {
        let vouch = Use { limb: "kern".into(), path: vec!["vouch".into()], thing: None, doing: "vouches with `!check`".into(), at: 0 };
        let r = check(&vouch, &limb("kern", &[Power::new("vouch")]), &[]).unwrap_err();
        assert!(matches!(r, Refusal::NotGranted { .. }));
        // IEG listed: granted to every limb built with it.
        let mut k = limb("kern", &[Power::new("vouch")]);
        k.grants.push(Grant { power: Power::new("vouch"), by: "IEG".into(), to: None });
        assert_eq!(check(&vouch, &k, &[]), Ok(()));
    }

    #[test]
    fn deny_with_except_and_for() {
        let l = limb("fastjson", &[Power::new("c_calls")]);
        let no_c_but_sqrt = Deny { power: Power::new("c_calls"), except: vec!["sqrt".into()], for_limbs: vec![] };
        assert!(matches!(check(&c_call("fastjson", "write"), &l, &[no_c_but_sqrt.clone()]), Err(Refusal::Denied { .. })));
        assert_eq!(check(&c_call("fastjson", "sqrt"), &l, &[no_c_but_sqrt]), Ok(()));
        let only_for_other = Deny { power: Power::new("c_calls"), except: vec![], for_limbs: vec!["other".into()] };
        assert_eq!(check(&c_call("fastjson", "write"), &l, &[only_for_other]), Ok(()));
    }

    #[test]
    fn a_favour_reaches_only_the_ids_it_names() {
        let mut l = limb("fav", &[Power::new("operators_on_builtin_types")]);
        l.grants.push(Grant { power: Power::new("operators_on_builtin_types"), by: "IEG".into(), to: Some(vec!["abc".into()]) });
        let op = Use { limb: "fav".into(), path: vec!["operators_on_builtin_types".into()], thing: None, doing: "defines `op mean` on a `bin64`".into(), at: 0 };
        assert!(matches!(check(&op, &l, &[]), Err(Refusal::NotGranted { .. })));
        l.id = Some("abc".into());
        assert_eq!(check(&op, &l, &[]), Ok(()));
    }
}
