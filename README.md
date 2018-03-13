# Sanskrit
A Total smart contract virtual machine with affine type support.
This is the first part of my PhD project at the University of Zurich related papers and research proposal links will follow soon.

## License
Copyright 2018 Markus Knecht, System Communication Group, University of Zurich.
Concrete Licence is not yet defined.

## Status
This is very early state and currently just a description on what to the goal is with no commited code yet.

## Why another smart contract virtual machine
Most currently used smart contract virtual machine couple the code and the state of a contract/blockchain object tightly. The code associated with a contract is the only code that can write and read to/from the associated storage/state. In exchange, this code can be granted arbitrary access to its storage and other local resources like its memory or stack and thus can be a simple low-level bytecode without a type system or any enforced guarantees. This model is simple and easy to implement but has some drawbacks as well. It is not possible to do cross contract optimisations like for example inlining a cross-contract call and further it is not possible to compile languages to it that require that certain guarantees given by the compiler hold at runtime as soon as these guarantees have to hold cross contracts. This prevents the use of alternative concepts and paradigms that may be a benificial addition to smart contract programming. With Sanskrit VM and later a high-level language (codename: Mandala) such alternative concepts and paradigms will be explored.

## What different about the Sanskrit virtual machine
The Sanskrit virtual machine does not only include a low-level bytecode interpreter but additionally a compiler that compiles a mid-level code representation into low-level bytecode. High-level languages do produce the mid-level code which then is compiled to low-level code. This compilation is part of the blockchain consensus and only low level code produced by this compilation step is deployed to the blockchain. This allows having a type system as well as certain cross contract guarantees in the mid-level code that can not be circumvented at. Thanks to optimisations during the onchain compilation the runtime overhead can be kept near zero and often be eliminated completely. It does further allow to have optimisations like for example cross contract inlining. 

## What are the design decisions and features of the Sanskrit virtual machine

### Static Dispatch
The Sanskrit virtual machine does not have any kind of dynamic dispatch making reasoning about code easier as tools and auditors can always be certain what code is executed on a call and further allows aggressive inlining which reduces expensive disk reads for looking up the called function during execution and further reduces the number of necessary proofs if a stateless client model would be used. This model is, also, easier to be used in junction with formal verification compared to a model that includes dynamic dispatches.

### Total Language
The Sanskrit virtual machine is not Turing complete as it does not allow recursive function calls and does only allow loops with an upper bound of iterations. This does make it possible to calculate an upper cost of the resources consumed during a function call allowing to design alternative gas models that can never run out of gas by requiring the caller to have enough reserves to pay for the worst case execution path. 

### Algebraic Datatypes
The Sanskrit virtual machine is founded on immutable non-recursive algebraic datatypes as its fundamental representation of values giving it a functional touch with all the benefits coming from that. Sanskrit algebraic datatypes have some special properties that make them especially well suited for programming smart contracts in a way different from current approaches and idiomatic to Sanskrit.

#### Authentic Opaque Types
The Sanskrit types have by default two fundamental properties differentiating them from classical algebraic datatypes. First, only code belonging to the same Module (Sanskrit deployment unit) as the type can create values of that type (Authentic) and access the fields inside the type (Opaque). This ensures a holder of a value that it was constructed under certain circumstances dictated by the Module containing the type. If it is wished that a field of a type can be read by other Modules, the type can be declared Transparent. Further, a type can be declared Basic, allowing other Modules to create instances (but not read the fields, except it is transparent as well)

#### Affine Types
Beside Basic and Authentic (the default) a Sanskrit type can be declared to be Affine which has all the benefits of Authentic but the compiler does additionally enforce that a value of this type once created cannot be duplicated. Meaning that a function receiving a value of an affine type can use this value at most once. This makes affine types the perfect candidate for representing assets, tokens, cryptocurrencies etc... and thus the Sanskrit virtual machine does not have a native cryptocurrency that it must treat specially as it can conveniently be represented on the type level with the existing concepts.

### Generics
The Sanskrit virtual machine does support Generic Functions and Types meaning that a Type or Function can take another type as a parameter and thus can be defined in a more general type independent way. To interact with the behaviour system (Basic, ... , Affine) generic parameters have additional markers on functions that declare the most restrictive behaviour they support. A generic algebraic datatype doe increae its behavioural restriction if instantiated with a more restrictive type then itself.

### Capabilities
The type system of Sanskrit is powerful enough to provide a capability-based access control system that has near zero runtime overhead and allows to check access control during compilation and thus code accessing values or calling functions it is not allowed to does not  compile.

### Cells and References
Cells and References are Sanskrit way of providing persisted state. Every type can have an initialisation function that can be used to generate a cell containing an initial value of that type. The creator does only receive a reference to the cell. This reference has the same behaviour as an authentic type and thus inherently encodes a capability giving access to the cell. A normal cell can only be modified and read by code from the same Module as the type of the cell. If the type is transient other Modules can read the value in the cell. References to cells can be wrapped (or unwrapped) into special wrapper types (types with exactly one constructor and exactly one field) allowing to provide different views on to the value (to create or drop a wrapper the normal rules are followed in respect of creating an instance or reading fields). This allows to encode further capabilities (or drop them) into the reference and thus program by the principle of Least Authority. Besides creating new instances, cells can be used as a global map, mapping a set of arguments (including generics) to references.

### Sharding Aware
The Sanskrit virtual machine is designed in a way that has certain benefits in a sharded environment. This includes that Module code is immutable and stateless and once the code is compiled and deployed it will always be their and has the same functionallity independent of the active shard. This means that one shard could use code deployed on another shard or on a dedicated deployment shard / chain. This makes it possible that values can be transferred between shards as the code that operates on them is available on other shards. Even references can be transferred between shards on each shard the reference initially points to a cell with the value containing the initial value (lazy initialised).

### Stateless Client Aware
The Sanskrit virtual machine is designed in a way that reduces the proof overhead in a stateless client model. Beside the already mentioned inlining references to other components that are not eliminated are represented as hashes of the targets content. This means that a proof to the  entry point of the transaction is sufficient to proof the existenz and validity of all code that can be executed during a function call. Further Sanskrits optimisations and overall paradigm encourages a programming style where state is manipulated over pure functions and only persisted into cells when needed. It Further allows to represent a lot of concepts like for example access contoll as pure type system concepts which do not need to access cells at all during runtime. If multiple cells are accessed, the proof may be slightly bigger in contrast to other smart contract vm's as related cells are not bundled under a common prefix (contract).

## Example Pseudo Code
Sanskrit requires a different programming style than other smart contract systems the following pseudocode should give a feel for how Sanskrit could look like. Most of the presented code probably would be in a standard library. The syntax just descriptional as real Sanskrit is a bytecode format

### Token
This Module represents a Generic Token used to represent all kind of Tokens. 

```
module Token {
  public affine transient type Token[T](Int)                          //T is the concrete token type as well as the minting capability
  public mint[affine T](capability:T, amount:Int) => Token[T](amount) //only the possesor of a value of T can mint
  public merge[affine T](Token[T](amount1), Token[T](amount2)) =>  Token[T](amount1 + amount2)
  public split[affine T](Token[T](amount), split:Int) => case sub(amount,split) of 
                                                    success(res) => Some((Token[T](res), Token[T](split)))
                                                    underflow => None
  public zero[affine T]() => Token[T](0)
  .... //Probably more stuff
}

module Purse {
  public affine type Purse[T]
  public transient type Owned[T](Purse[T])                  //Capability allowing to withdraw funds (In reality would use Cap see later)
  public init Purse[T] => Owned[T](Purse[T](Token.zero()))  //The creator is the Owner

  public deposit[affine T](purse: ref Purse[T], deposit:Token[T]) => modify purse with 
                                                                        Purse(t) => store Purse(Token.merge(t, deposit))
  public withdraw[affine T](purse: ref Owned[T], amount:Int) => modify purse with 
                                                                  Owned(Purse(t)) => case Token.split(t,amount) of
                                                                    Some((rem,split)) => store Owned(Purse(re)) and return split
                                                                    None => store Owned(Purse(t)) and return Token.zero()
}

module DefaultPurseStore{
  private cell purse[T](address:Address):Purse.Owned[T] //maps each T address pair to a reference to a Owned

  public getMyPurse[affine T]() => purse[T](this)   //this is the transaction initiator                                                 
  public getPurse[affine T](address:Address) => purse[T](address).unwrap[Purse[T]] //unwrap removes the Owned capability
  public transfer[affine T](target:Address, amount:Int) => getPurse[T](target).deposit(Purse[T].withdraw(getMyPurse[T](), amount)) 
  
  .... //Probably some more stuff
}

```

#### Token Instantiation
```
import Token;     //In reality, the hash of the Token Module, Sanskrit is content addressed
module MyFixSupplyToken {
    public type MyToken
    //MyToken instance is capability allowing to create Token[MyToken]
    on deploy => DefaultPurseStore.getMyPurse().deposit(Token.mint[MyToken](MyToken, 100000000)) 
}
```
### Generic Capabilities
```
module Capability {
  public transient type Cap[C,T](ref T)
  public addCap[affine C,affine T](capability:C, value:ref T) => Cap[C,T](value)
  public combineCap[affine C1,affine C2,affine T](Cap[C1,T](ref val1), Cap[C2,T](ref val2)) => case val1 == val2 of
                                                                                                  True => Some(Cap[(C1,C2),T](val))
                                                                                                  False => None
  public splitCap[affine C1,affine C2,affine T](Cap[(C1,C2),T](ref val)) => (Cap[C1,T](val), Cap[C2,T](val))
  
  .... //probably much more
}

```

### Some Virtual Crypto
```
//Virtual encryption
module Sealed {
  public affine type Sealed[F,T](T)
  public sealFor[affine F,affine T](val:T) => Sealed(val)
  public unseal[affine F,affine T](capability:F, Sealed[F,T](val)) => val //only possesor of capability F can unseal
}
//Virtual Signature
module Authenticated {
  public transient affine type Signed[S,T](T)
  public sign[affine S,affine T](capability:S, val:T) => Signed[S,T](val) //only possesor of capability S can sign
}
```
