const exec = require('child_process').exec;
const fs = require('fs');

let map = {};

let files = fs.readdirSync('test_cases/');
for (file of files) {
    if (!map[/(.+)\..+/g.exec(file)[1]]) map[/(.+)\..+/g.exec(file)[1]] = new Array(3);
    if (file.endsWith('.arsm')) {
        map[file.slice(0, file.length - 5)][0] = 'test_cases/' + file;
    } else if (file.endsWith('.in')) {
        map[file.slice(0, file.length - 3)][1] = 'test_cases/' + file;
    } else if (file.endsWith('.out')) {
        map[file.slice(0, file.length - 4)][2] = fs.readFileSync('test_cases/' + file).toString();
    }
}
for (let c in map) {
    c = map[c];
    exec(`cargo -q run ${c[0]} --stdin ${c[1]}`,  (_, stdout) => {
        if (stdout == c[2]) {
            console.log("Testcase " + c[0] + ": \x1b[32m Passed\x1b[0m");
        } else {
            console.log("Testcase " + c[0] + ": \x1b[31m Failed\x1b[0m");
            console.log("  Expected: " + c[2]);
            console.log("  Got:      " + stdout);
        }
    });
}
