use std::path::{Path, Component};
use pulldown_cmark::{html, Options, Parser};
use std::collections::HashMap;
use std::default::Default;
use std::fs;
use std::fs::read_dir;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use tera::Context;
use tera::Tera;
use tera::Value;
use std::ffi::OsStr;

fn main() {
    println!("cargo:rerun-if-changed=statics/*");
    proccess_statics();
    merry_war();
}

fn proccess_statics() {
    let out_dir = PathBuf::from("built_statics");
    let tmp_dir = PathBuf::from("statics/tmp");
    match fs::create_dir(&out_dir) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
        Err(e) => panic!("{}", e),
    }
    match fs::create_dir(&tmp_dir) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
        Err(e) => panic!("{}", e),
    }
    let mut tera = Tera::default();
    initialize_tera(&mut tera);
    fn files_of_dir<P: AsRef<Path>>(path: P) -> Vec<PathBuf> {
        read_dir(path)
            .expect("Could not read dir")
            .map(|r| r.expect("Bad Direntry"))
            .map(|d| d.path())
            .filter(|p| !p.to_str().unwrap().contains(&"#"))
            .filter(|p| !p.to_str().unwrap().ends_with(&"~"))
            .collect::<Vec<PathBuf>>()
    }
    let mut files = files_of_dir("statics");
    let mut defered: Vec<PathBuf> = vec![];
    loop {
        let mut made_progress = false;
        for path in &files {
            if path == &tmp_dir {
                continue;
            }
            if path.is_dir() {
                defered.extend(files_of_dir(path));
                made_progress = true;
                continue;
            }

            eprintln!("About to handle path {:?}", &path);
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
                            let mut new_path = tmp_dir.clone();
                            for component in path.with_extension("").components().skip(1){
				new_path.push(component);
			    }
                            dbg!(&new_path);
                            let new_path = new_path;
			    fs::create_dir_all(new_path.parent().unwrap()).unwrap();
                            if path == &PathBuf::from("statics/index.html.tera") {
                                assert_eq!(new_path, PathBuf::from("statics/tmp/index.html"));
                            }
                            let rendered = tera
                                .render(name, &Context::new())
                                .expect("Error During rendering");
                            let mut file =
                                File::create(&new_path).expect("Error opening target of render");
                            write!(file, "{}", rendered)
                                .expect("Error writing to target of render");
                            defered.push(new_path);
                        }
                        Err(e) => panic!("{:#?}", e),
                    }
                }
                _ => {
                    println!("{:?}", path);
		    let mut out_path = out_dir.clone();
		    for component in path.components().skip(1) {
                        if component == Component::Normal(OsStr::new("tmp")) {
			    continue
			}
			out_path.push(component);
		    }
		    dbg!(&out_path);
		    fs::create_dir_all(out_path.parent().unwrap()).unwrap();
                    fs::copy(&*path, out_path)
                        .expect("Copying a file failed");
                    made_progress = true;
                }
            }
        }
        dbg!(&defered);
        if defered.is_empty() {
            break;
        }
        if !made_progress {
            panic!("Unhandled paths: {:?}", files)
        }
        files = defered.clone();
        defered = Vec::new();
    }
}

fn initialize_tera(tera: &mut Tera) {
    tera.register_filter("markdown", markdown_filter);
}

fn markdown_filter(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let options = Options::empty();

    let parser = Parser::new_ext(value.as_str().unwrap(), options);

    let mut output = String::new();
    html::push_html(&mut output, parser);
    Ok(Value::String(output))
}


fn merry_war() {
    use std::process::Command;
    
    let status = Command::new("inklecate")
        .arg("-j")
        .arg("merry_war/merry_war.ink")
        .output()
        .expect("Failed to compile merry_war");

    println!("{}", status);
    
    // assert_eq!(status, r#"{"compile-success": true}
    // {"issues":[]}{"export-complete": true}"#);

    let json = std::fs::read("merry_war/merry_war.ink.json").expect("ink compileation failed");
    std::fs::create_dir("built_statics/merry_war");
    let mut js_file = File::create("built_statics/merry_war/merry_war.js").expect("creating merry war js file failed");

    js_file.write(b"var storyContent = ").expect("writing to merry_war.js failed");
    js_file.write(&json).expect("wriging json failed");

    for file in &["index.html", "main.js", "ink.js", "style.css"] {
        std::fs::copy(&format!("merry_war/{}", file), &format!("built_statics/merry_war/{}", file)).expect("copying merry war file failed");
    }

}
