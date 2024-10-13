# Torq Lang

Welcome to the Torq Programming Language for Rust developers.

Big Data analytics, artificial intelligence, and IT/OT convergence are pressuring organizations to be data-centric. The urgent need to manage explosive data growth with meaning, where billions of files and records are distributed over thousands of computers, is a significant challenge.

Torq addresses these challenges by simplifying data-centric programming at scale with a novel programming model. This allows programmers to focus on the application problem rather than the programming problem, leading to efficiencies that lower costs.

The Torq programming model is designed to improve scalability, responsiveness, and faster time-to-market in three problem areas:

1. Data contextualization - enhancing live data meaningfully before it is stored
2. Enterprise convergence - creating meaningful APIs across departments and divisions
3. Situational applications - composing low-cost, low-code, highly-valued applications quickly

## Actorflow

Torq is a new programming language based on *Actorflow*, a patented programming model that fuses message-passing actors with a hidden implementation of declarative dataflow. Concurrently executing actors only communicate by sending immutable messages. Requests and responses are correlated with private dataflow variables, bound indirectly by a controller. Actors that send requests may suspend, waiting for a variable bound by a response. Actors that receive messages may resume when a received message binds a waiting variable. This request-response interaction provides synchronization without sharing variables, giving us a naturally sequential programming style. Moreover, we can compose programs using a mix of libraries from other programming languages. All variables, dataflow or otherwise, are hidden.

Consider the following program written as a Torq actor. `ConcurrentMath` calculates the number `7` using three concurrent child actors to supply the operands in the expression `1 + 2 * 3`. This example is an unsafe race condition in mainstream languages. However, in Torq, this sequential-looking but concurrently executing code will always calculate `7` because of the dataflow rule defined previously. Notice that Torq honors operator precedence without explicit synchronization.

```
actor ConcurrentMath() in
    actor Number(n) in
        handle ask 'get' in
            n
        end
    end
    var n1 = spawn(Number.cfg(1)),
        n2 = spawn(Number.cfg(2)),
        n3 = spawn(Number.cfg(3))
    handle ask 'calculate' in
        n1.ask('get') + n2.ask('get') * n3.ask('get')
    end
end
```

## Concurrent Construction

Torq facilitates a concurrent style of programming not possible in mainstream languages. Consider the next example as a slightly modified version of our previous example. Instead of *calculating* concurrently, we *construct* concurrently. The concurrent math calculation `x + y * z` from our first example is replaced with a concurrent data construction `[x, y, z]`, where `x`, `y`, and `z` stand for `n1.ask('get')`, `n2.ask('get')`, and `n3.ask('get')`, respectively.

```
actor ConcurrentMathTuple() in
    actor Number(n) in
        handle ask 'get' in
            n
        end
    end
    var n1 = spawn(Number.cfg(1)),
        n2 = spawn(Number.cfg(2)),
        n3 = spawn(Number.cfg(3))
    handle ask 'calculate' in
        [n1.ask('get'), n2.ask('get'), n3.ask('get')]
    end
end
```

Dataflow variables make concurrent construction possible. Instead of using futures (functions and callbacks), like other programming languages, Torq uses dataflow variables to construct partial data that is complete when concurrent tasks are complete. In essence, futures wait for logic, but Torq waits for data.
