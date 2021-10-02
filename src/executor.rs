use std::fs;
use std::fs::File;
use std::str;

use tokio::process::Command;

use crate::{CodeExecutionResult, CodeToExecute};

pub(crate) async fn execute_code(program: CodeToExecute) {
    println!("Compiling and executing code! {}", program.code);
    let target_dir = "/var/www/archibald/programs/".to_string() + &*program.id.to_string();
    fs::create_dir_all(&target_dir);

    let mut file = File::create(target_dir + "/submission.cpp");

    println!("Initializing isolate");
    Command::new("isolate").arg("--cleanup").output().await;
    Command::new("isolate").arg("--init").output().await;


    // compile with: isolate --run -p --dir=/a=/var/www/a  -- /usr/bin/g++ /a/b.cpp -o out
    println!("Compiling...");
    let output = Command::new("isolate")
        .args(&[
            "--run",
            "-p",
            &("--dir=/code=/var/www/archibald/programs/".to_string() + &*program.id.to_string()),
            //"--dir=/usr/bin/ld=/usr/bin/",
            "--",
            "/usr/bin/g++",
            "-shared",
            "/code/submission.cpp",
            "-o",
            "out"])
        .output().await.expect("Compilation output");
    let compilation_stdout = match str::from_utf8(&output.stdout) {
        Ok(v) => v,
        Err(e) => "Invalid UTF-8 sequence"
    };
    let compilation_stderr = match str::from_utf8(&output.stderr) {
        Ok(v) => v,
        Err(e) => "Invalid UTF-8 sequence"
    };

    println!("Running code!");
    let run_output = Command::new("isolate").args(&["--run", "--", "out"]).output().await.expect("Run command output");
    let stdout = match str::from_utf8(&run_output.stdout) {
        Ok(v) => v,
        Err(e) => "Invalid UTF-8 sequence"
    };
    let stderr = match str::from_utf8(&run_output.stderr) {
        Ok(v) => v,
        Err(e) => "Invalid UTF-8 sequence"
    };

    let result = CodeExecutionResult {
        successful: output.status.success(),
        stdout: Box::from(stdout),
        stderr: Box::from(stderr),
        compilation_stdout: Box::from(compilation_stdout),
        compilation_stderr: Box::from(compilation_stderr),
    };
    println!("Result done, sending...");
    program.oneshot_sender.send(result);
}

