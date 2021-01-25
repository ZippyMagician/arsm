const exec = require('child_process').exec;
const fs = require('fs');

const FILE_EXTENSION = ".asm";

function testPath(path) {
    let map = {};

    let files = fs.readdirSync(path);
    for (file of files) {
        if (file.indexOf('.') > -1) {
            let ident = /(.+)\..+/g.exec(file)[1];

            if (!map[ident]) map[ident] = new Array(3);
            if (file.endsWith(FILE_EXTENSION)) {
                map[ident][0] = 'test_cases/' + file;
            } else if (file.endsWith('.in')) {
                map[ident][1] = fs.readFileSync('test_cases/' + file).toString();
            } else if (file.endsWith('.out')) {
                map[ident][2] = fs.readFileSync('test_cases/' + file).toString();
            }
        }
    }


    for (let c in map) {
        c = map[c];
        exec(`cargo -q run "${c[0]}" -u "${c[1]}"`,  (err, stdout, stderr) => {
            if (stdout == c[2]) {
                console.log("Case " + c[0] + ": \x1b[32m Passed\x1b[0m");
            } else {
                console.log("Case " + c[0] + ": \x1b[31m Failed\x1b[0m");
                console.log("  Expected: " + c[2]);
                console.log("  Got:      " + stdout);
                console.log("  Errors: " + stderr);

                process.exit(1);
            }
        });
    }
}

// Python test cases are not run here
testPath('test_cases/');