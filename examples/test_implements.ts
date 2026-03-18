interface Named {
    name: string,
}

class Person implements Named {
    name: string,
    age: number,
}

function main(): void {
    println("class implements interface works!");
}
