use std::default::Default;
use std::fs;
use std::fs::read_dir;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use tera::Context;
use tera::Tera;

fn main() {
    let out_dir = PathBuf::from("built_statics");
    let mut tera = Tera::default();
    let mut files = read_dir("statics")
        .expect("Could not read statics dir")
        .map(|r| r.expect("Bad Direntry"))
        .map(|d| d.path())
        .filter(|p| !p.to_str().unwrap().contains(&"#"))
        .filter(|p| !p.to_str().unwrap().ends_with(&"~"))
        .collect::<Vec<PathBuf>>();
    let mut defered: Vec<PathBuf> = vec![];
    loop {
        let mut made_progress = false;
        for path in &files {
            match path
                .extension()
                .expect("All static files should have an extension")
                .to_str()
                .unwrap()
            {
                "tera" => {
                    let name = path
                        .file_name()
                        .expect("tera files should always have a file name")
                        .to_str()
                        .unwrap();
                    match tera.add_template_file(path, Some(name)) {
                        Err(err)
                            if match err.kind {
                                tera::ErrorKind::MissingParent { .. } => true,
                                _ => false,
                            } =>
                        {
                            defered.push(path.to_path_buf())
                        }
                        Ok(()) => {
                            made_progress = true;
                            let new_path = path.with_extension("");
                            let rendered = tera
                                .render(name, &Context::new())
                                .expect("Error During rendering");
                            let mut file =
                                File::create(new_path).expect("Error opening target of render");
                            write!(file, "{}", rendered).expect("Error writing to target of render");
                        }
                        Err(e) => panic!("{:#?}", e),        
                    }
                }
                _ => {
                    println!("{:?}", path);
                    fs::copy(&*path, out_dir.join(path.file_name().unwrap()))
                        .expect("Copying a file failed");
                    made_progress = true;
                }
            }
        }
        if defered.is_empty() {
            break;
        }
        if !made_progress {
            panic!("Unhandled paths: {:?}", files)
        }
        files.extend(defered);
        defered = Vec::new();
    }
}
