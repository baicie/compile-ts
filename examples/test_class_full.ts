interface Animal {
    name: string,
    age: number,
}

class Person implements Animal {
    name: string,
    age: number,
}

function main(): void {
    let p: Person = new Person();
    p.name = "Alice";
    p.age = 25;
    
    println("Name: ");
    println(p.name);
    println("Age: ");
    println(p.age);
}
