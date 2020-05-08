use std::io;
use std::default::Default;
use std::fs;
use std::fs::read_dir;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tera::Context;
use tera::Tera;
use tera::Value;
use std::collections::HashMap;
use pulldown_cmark::{Parser, Options, html};

fn main() {
    println!("cargo:rerun-if-changed=statics/*");    
    proccess_statics()
}

fn proccess_statics() {
    let out_dir = PathBuf::from("built_statics");
    let tmp_dir = PathBuf::from("statics/tmp");
    match fs::create_dir(&out_dir){
	Ok(()) => {}
	Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
	Err(e) => panic!("{}",e)
    }
    match fs::create_dir(&tmp_dir){
	Ok(()) => {}
	Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
	Err(e) => panic!("{}",e)
    }
    let mut tera = Tera::default();
    initialize_tera(&mut tera);
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
	    if path == &tmp_dir { continue }
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
			    let new_file_name = path.with_extension("");
			    dbg!(&new_file_name);
			    new_path.push(new_file_name.file_name().unwrap());
			    let new_path = new_path;
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
                    fs::copy(&*path, out_dir.join(path.file_name().unwrap()))
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

fn markdown_filter(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value>{
    let options = Options::empty();

    let parser = Parser::new_ext(value.as_str().unwrap(), options);

    let mut output = String::new();
    html::push_html(&mut output, parser);
    Ok(Value::String(output))
}
