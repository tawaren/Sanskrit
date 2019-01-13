# Sanskrit
A Total smart contract virtual machine with opaque and affine type support.
This is the first part of my PhD project at the University of Zurich. 

## License
Copyright 2018 Markus Knecht, System Communication Group, University of Zurich.
Concrete Licence is not yet defined.

## Status
Most of the functionlity is finished some things have still to be tested and the metering has to be finished

## Why another smart contract virtual machine
Most currently used smart contract virtual machine couple the code and the state of a contract/blockchain object tightly. 
The code associated with a contract is the only code that can write and read to/from the associated storage/state. 
In exchange, this code can be granted arbitrary access to its storage and other local resources like its memory or stack and thus can be a simple low-level bytecode without a type system or any enforced guarantees.
This model is simple and easy to implement but has some drawbacks as well. 
It is not possible to do cross contract optimisations like for example inlining a cross-contract call and further it is not possible to compile languages to it that require that global guarantees given by the compiler hold at runtime. 
This prevents the use of alternative concepts and paradigms that may be a beneficial addition to smart contract programming. 
With Sanskrit VM and later a high-level language (codename: Mandala) such alternative concepts and paradigms can be used and will be exlored.

## What is different about the Sanskrit virtual machine
The Sanskrit virtual machine does not only consist of a low-level bytecode interpreter but additionally a compiler that compiles a mid-level code representation into low-level bytecode. 
High-level languages do produce the mid-level code which then is compiled to low-level code. 
This mid to low-level compilation is part of the blockchain consensus and only low-level code produced by this compilation step is deployed to the blockchain. 
This allows having a type system as well as certain cross contract guarantees in the mid-level code that can not be circumvented by any high-level language. 
Thanks to optimisations during the on-chain compilation the runtime overhead to ensure such guarantees can be kept near zero and often be eliminated completely. 
It does further allow to have optimisations like for example cross contract inlining that would not be possible otherwise. 

## What is the goal of the Sanskrit virtual machine
The goal of the Sanskrit virtual machine is to explore new concepts and paradigms that are rarely if at all used in other language and evaluate if they provide a benefit for smart contract programming. 
The assumption behind this approach is that the smart contract programming environment is different enough to the classical (non-smart contract) programming enviroments such that it is plausible that approaches that are inappropriate in the later may be viable and benefical in the former.

## What are the design decisions and features of the Sanskrit virtual machine

### Static Dispatch
The Sanskrit virtual machine does not have any dynamic function dispatches making reasoning about code easier. 
Tools and auditors can always be certain what code is executed on a call. 
This further allows aggressive inlining which reduces expensive disk reads needed for looking up the called functions code during execution and further reduces the number of necessary proofs if a stateless client model would be used. 
This model is, also, easier to be used in junction with formal verification compared to a model that includes dynamic dispatches.

### Bounded time Language
The Sanskrit virtual machine is not Turing complete as it does not allow recursive function calls or loops.
Together with the absence of dynamic dispatches this does make it possible to calculate an upper bound on the resources consumed during a function call allowing to design alternative gas models that can never run out of gas by requiring the caller to have enough reserves to pay for the worst case execution path. 
It is still possible to only charge for the executed path

### Algebraic Datatypes
The Sanskrit virtual machine is founded on immutable non-recursive algebraic datatypes as its fundamental representation of values giving it a functional touch with a lot of the benefits coming from that. 
Sanskrit algebraic datatypes have some special properties that make them especially well suited for programming smart contracts in a way different from current approaches and idiomatic to Sanskrit.

#### Opaque Types
The Sanskrit types do by default restrict the interaction possibilitties for functions with values of that type considerably. 
By default functions cannot create values, access fields, copy values, discard values or persist values. 
When declaring a type, these restriction can individually be removed by granting priviledges resulting in a finetuned type that provides the needed behaviour. 
Some of the privileges (copy, deiscard and persist) are recursive, meaning that an algebraic data type can only have these privileges if the parameters of all constructor fields have them as well. 
Functions that are defined in the same Module as the type are always treated as if they have the non-recursive privileges even if the type declaration does not grant these privileges.
For Example a type with access, discard and perstsit priviliges make the perfect candidate for representing assets, tokens, cryptocurrencies etc... and thus the Sanskrit virtual machine does not have a native cryptocurrency that it must treat differently as they can conveniently be represented with the existing concepts. 

### Generics
The Sanskrit virtual machine does support generic functions and types meaning that a type or function can take other types as parameters and thus can be defined in a more type independent way. 
To interact with the restriction system, generic parameters on functions must declare if they require additional priviledges preventing the caller to instantiate them with a type that does not grant the requested privileges. 
If a type parameter in an algebraic datatype is less priviledged in respect to its recursive priviledges then the resulting types priviledges are reduced accordingly. 
A type parameter can be marked phantom and in that case it can not be used as a type for a constructor parameter and can only used as generic parameter to other phantom type parameters, in return phantom generic parameters never strip privileges away from the resulting type.

### Capabilities
The type system of Sanskrit is powerful enough to provide a capability-based access control system that has near zero runtime overhead and allows to check access control during compilation and thus code accessing values or calling functions, that it is not allowed to does not compile.

### Error Handling
Saskrit functions can raise an Errpr but these Errors have to be declared to make callers aware of the potential Error.
These Errors can be captured over a try catch statement.
A Module in Sanskrit can declare custom Error codes.

## Example Pseudo Code
Sanskrit requires a different programming style than other smart contract systems the following pseudocode should give a feel for what Sanskrit can do and how it achieves it. 
Most of the presented code probably would be in a standard library. 
The syntax is just descriptional as real Sanskrit is a bytecode format. 
The used syntax is inspired by the vision for the future high-level language Mandala that will compile to Sanskrit bytecode.

### Token

TODO
