extern crate cargo;
extern crate walkdir;

macro_rules! bail {
    ($($fmt:tt)*) => (
        return Err(human(&format_args!($($fmt)*)))
    )
}

pub mod ops {
    use std::path::{Path, PathBuf};
    use std::fs;
    use std::env;

    use cargo::util::{CargoResult, Config, human};
    use cargo::util::to_semver::ToSemver;
    use cargo::core::package_id::PackageId;
    use cargo::core::source::{Source, SourceId};
    use cargo::core::registry::Registry;
    use cargo::core::dependency::Dependency;
    use cargo::sources::RegistrySource;

    use walkdir::{DirEntry, WalkDir, WalkDirIterator};

    pub fn clone(krate: &Option<String>,
                 srcid: &SourceId,
                 flag_version: Option<String>,
                 config: Config)
                 -> CargoResult<()> {

        let krate = match *krate {
                Some(ref k) => k,
                None => bail!("specify which package to clone!"),
        };

        let mut regsrc = RegistrySource::new(&srcid, &config);
        try!(regsrc.update());

        let version = match flag_version {
            Some(v) => {
                match v.to_semver() {
                    Ok(v) => v,
                    Err(e) => bail!("{}", e),
                }

            },
            None => {
                let dep = try!(Dependency::parse(krate, flag_version.as_ref().map(|s| &s[..]), &srcid));
                let summaries = try!(regsrc.query(&dep));

                let latest = summaries.iter().max_by_key(|s| s.version());

                match latest {
                    Some(l) => l.version().to_semver().unwrap(),
                    None => bail!("package '{}' not found", krate),
                }
            },
        };

        let pkgid = try!(PackageId::new(&krate, version, srcid));

        try!(regsrc.download(&[pkgid.clone()]));

        let crates = try!(regsrc.get(&[pkgid.clone()]));
        let mut dest_path = try!(env::current_dir());
        dest_path.push(krate);

        try!(clone_directory(crates[0].root(), &dest_path));

        Ok(())
    }

    fn clone_directory(from: &Path, to: &Path) -> CargoResult<()> {
        try!(fs::create_dir(to));

        for entry in WalkDir::new(from) {
            let entry = entry.unwrap();
            //try!(fs::copy(from, to));
            println!("{}", entry.path().display());
        }

        Ok(())
    }

}