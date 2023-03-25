import { Pyboard } from './index.js'

try {
    const pyboard = new Pyboard("/dev/pts/4", 115200)
    for (const file of pyboard.ls('.')) {
        console.log(file)
    }

    console.log(pyboard.exists('Makefile'))
    console.log(pyboard.cat('Makefile'))

} catch(error) {
    console.log(error)
}