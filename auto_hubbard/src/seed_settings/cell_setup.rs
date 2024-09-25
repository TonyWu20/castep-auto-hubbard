use std::{
    fs::{read_to_string, write},
    path::Path,
};

use regex::{Captures, Regex};

use super::JobType;

pub fn hubbard_before<P: AsRef<Path>>(
    init_hubbard_u: f64,
    u_step: i32,
    job_type: JobType,
    cell_file: P,
) -> Result<(), std::io::Error> {
    let i_u_value = init_hubbard_u + u_step as f64;
    let (u_value, alpha_value) = match job_type {
        JobType::U => (i_u_value, init_hubbard_u),
        JobType::Alpha => (init_hubbard_u, i_u_value),
    };
    let cell_content = read_to_string(cell_file.as_ref())?;
    let cell_content = cell_content.replace("\r\n", "\n");
    let replace_curr_u_regex = Regex::new(r"([spdf]):.*").unwrap();
    let curr_u_replaced = replace_curr_u_regex
        .replace_all(&cell_content, |caps: &Captures| {
            format!("{}: {:.15}", &caps[1], u_value)
        })
        .to_string();
    // (?ms) sets flags m and s, which enable the multiline 19 and
    // dot_matches_new_line 10 modes, respectively.
    let hubbard_u_re = Regex::new(r"(?ms)\%BLOCK HUBBARD_U\s+(.+?)%ENDBLOCK HUBBARD_U").unwrap();
    let Some(caps) = hubbard_u_re.captures(&curr_u_replaced) else {
        eprintln!("No match");
        return Ok(());
    };
    let hubbard_alpha_re = Regex::new(r"_U").unwrap();
    let hubbard_alpha = hubbard_alpha_re.replace_all(&caps[0], "_ALPHA").to_string();
    let hubbard_alpha_value = Regex::new(":.*")
        .unwrap()
        .replace_all(&hubbard_alpha, format!(": {:.15}", alpha_value))
        .to_string();
    let new_cell_context = [curr_u_replaced, hubbard_alpha_value].join("\n");
    let new_cell_file = cell_file.as_ref().with_extension("bak");
    write(&new_cell_file, new_cell_context)?;
    #[cfg(debug_assertions)]
    {
        println!(
            "New cell content has been written to {}",
            new_cell_file.display()
        );
        Ok(())
    }
    #[cfg(not(debug_assertions))]
    {
        rename(new_cell_file, cell_file.as_ref())
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::seed_settings::JobType;

    #[test]
    fn hubbard_before() {
        let cwd = env!("CARGO_MANIFEST_DIR");
        let cell_path = Path::new(cwd)
            .parent()
            .unwrap()
            .join("sh/test/GDY_111_Fe_U.cell");
        super::hubbard_before(0.000000010000000, 2, JobType::U, &cell_path).unwrap();
        super::hubbard_before(0.000000010000000, 2, JobType::Alpha, &cell_path).unwrap();
    }
}
