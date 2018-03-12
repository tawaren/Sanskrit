# Sanskrit
A Total Smart Contract Virtual Machine with Affine type support.
This is my PhD project related papers and research proposal links will follow soon.

# License
Copyright 2018 Markus Knecht, System Communication Group, University of Zurich.
Concrete Licence has to be defined

## Status
This is very early state and just the description on what to the goal is with no commited code yet.

## Why another smart contract virtual machine
Most currently used smart contract virtual machine couple the code and the state of a contract/blockchain object tightly. The code associated with a contract is the only code that can write and read to/from the associated storage/state. In exchange, this code can be granted arbitrary access to its storage and other local resources like its memory or stack and thus can be a simple low-level bytecode without a type system or any enforced guarantees. This model is simple and easy to implement but has some drawbacks as well. It is not possible to do cross contract optimisations like for example inlining a cross-contract call and further it is not possible to compile languages to it that require that certain guarantees given by the compiler hold at runtime as soon as these guarantees have to hold cross contracts.

## What different about the Sanskrit virtual machine
The Sanskrit virtual machine does not only include a low-level bytecode interpreter but additionally a compiler that compiles a mid-level code representation into low-level bytecode. High-level languages do produce the mid-level Code which then is compiled to low-level code. This compilation is part of the blockchain consensus and only low level code produced by this compilation step is deployed to the blockchain. This allows having a type system as well as certain cross contract guarantees in the mid-level code that can not be circumvented at runtime without introducing a runtime overhead. It does further allow to have optimisations like for example cross contract inlining. 

## What are the design decisions and features of the Sanskrit virtual machine

### Static Dispatch
The Sanskrit virtual machine does avoid dynamic dispatch making reasoning about code easier as tools and auditors can always be certain what code is executed on a call and further allows aggressive inlining which reduces expensive disk reads for looking up the called function during execution and further reduces the number of necessary proofs if a stateless model is used. This model is, also, easier to do formal verification on compared to a model that includes dynamic dispatches.

### Total Language
The Sanskrit virtual machine is not Turing complete as it does not allow recursive function calls and does only allow loops with an upper bound of iterations. This does make it possible to calculate an upper cost of the resources consumed during a function call allowing to design alternative gas models that can never run out of gas by requiring the caller to have enough money to pay for the worst case execution path. 

### Algebraic Datatypes
The Sanskrit virtual machine is founded on Immutable non-recursive Algebraic Data types as its fundamental representation of values giving it a functional touch with all the benefits coming from that. Sanskrit Algebraic data types have some special properties that make them especially well suited for programming smart contracts in a way different from current approaches and idiomatic to Sanskrit.

#### Authentic Opaque Types
The Sanskrit types have by default two fundamental properties differentiating them from classical Algebraic Datatypes. First, only code belonging to the same Module (Sanskrit deployment unit) as the type can create values of that type (Authentic) and access the fields inside the type (Opaque). This ensures a holder of a value that it was constructed under certain circumstances described by the Module containing the type. If it is wished that a field of a type can be read by other Modules, it can be marked transparent. Further, a type can be marked basic, allowing other Modules to create instances (but not read the fields, except it is transparent as well)

#### Affine Types
Beside Basic and Authentic (the default) a Sanskrit type can be declared to be Affine which has all the benefits of Authentic but the compiler does additionally enforce that a value of a type once created cannot be duplicated. Meaning that a function receiving a value of an affine type can use this value at most once. This makes Affine types the perfect candidate for representing assets, tokens, cryptocurrencies etc... and thus the Sanskrit virtual machine does not have a native cryptocurrency that it must treat specially as it can conveniently be represented on the type level.

### Generics
The Sanskrit virtual machine does support Generic Functions and Types meaning that a Type or Function can take another type as a parameter and thus can be defined in a more general type independent way.

### Capabilities
The type system of Sanskrit is powerful enough to provide a capability-based access control system that has near zero runtime overhead and allows to check access control during compilation and thus code accessing values it is not allowed to does not even compile.

### Cells and References
Cells and References are Sanskrit way of providing persisted state. Every type can have an initialisation function that can be used to generate a Cell containing an initial value of that type. The creator does only receive a reference to the cell. This reference has the same property as an Authentic type and thus inherently encode a capability giving access to the Cell. A normal Cell can only be modified and read by code from the same Module as the type of the Cell. If the type is transient other Modules can read the value in the Cell. References to Cells can be wrapped into special wrapper types (types with exactly one constructor and exactly one field) allowing to give different views on to the value. This allows to encode further capabilities into the Reference and thus program by the principle of Least Authority. Besides creating new instances, Cells can be used as a global Map, mapping set of arguments to references.

### Sharding Aware
The Sanskrit virtual machine is designed in a way that has certain benefits in a sharded environment. This includes that Module code is immutable and once the code is compiled and deployed on one shard it could be theoretically referenced and used from other shards (if the blockchain supports that). This makes it possible that values can be transferred between shards as the code that operates on them is available on other shards or can be deployed to other shards. Even references can be transferred between shards on each shard the reference initially points to a Cell with the value containing the initial value (lazy initialised).

## Example Pseudo Code
Sanskrit requires a different programming style than other smart contract systems the following pseudocode should give a feel for how Sanskrit could look like. Most of the presented code probably would be in a standard library. The syntax just descriptional as real Sanskrit is a bytecode format

### Token
This Module represents a Generic Token used to represent all kind of Tokens. 

```
module Token {
  public affine transient type Token[T](Int)                    //T is the concrete token type as well as the minting capability
  public mint[T](capability:T, amount:Int) => Token[T](amount)  //only the possesor of a value of T can mint
  public merge[T](Token[T](amount1), Token[T](amount2)) =>  Token[T](amount1 + amount2)
  public split[T](Token[T](amount), split:Int) => case sub(amount,split) of 
                                                    success(res) => Some((Token[T](res), Token[T](split)))
                                                    underflow => None
  public zero[T]() => Token[T](0)
  .... //Probably more stuff
}

module Purse {
  public affine type Purse[T]
  public transient type Owned[T](Purse[T])                  //Capability allowing to withdraw funds (In reality would use Cap see later)
  public init Purse[T] => Owned[T](Purse[T](Token.zero()))  //The creator is the Owner

  public deposit[T](purse: ref Purse[T], deposit:Token[T]) => modify purse with 
                                                                      Purse(t) => store Purse(Token.merge(t, deposit))
  public withdraw[T](purse: ref Owned[T], amount:Int) => modify purse with 
                                                                Owned(Purse(t)) => case Token.split(t,amount) of
                                                                    Some((rem,split)) => store Owned(Purse(re)) and return split
                                                                    None => store Owned(Purse(t)) and return Token.zero()
}

module DefaultPurseStore{
  private cell purse[T](address:Address):Purse.Owned[T] //maps each T address pair to a reference to a Owned

  public getMyPurse[T]() => purse[T](this)   //this is the transaction initiator                                                  
  public getPurse[T](address:Address) => purse[T](address).unwrap[Purse[T]] //unwrap removes the Owned capability
  public transfer[T](target:Address, amount:Int) => getPurse[T](target).deposit(Purse[T].withdraw(getMyPurse[T](), amount)) 
  
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
  public addCap[C,T](capability:C, value:ref T) => Cap[C,T](value)
  public combineCap[C1,C2,T](Cap[C1,T](ref val1), Cap[C2,T](ref val2)) => case val1 == val2 of
                                                                            True => Some(Cap[(C1,C2),T](val))
                                                                            False => None
  public splitCap[C1,C2,T](Cap[(C1,C2),T](ref val)) => (Cap[C1,T](val), Cap[C2,T](val))
  
  .... //probably much more
}

```

### Some Virtual Crypto
```
//Virtual encryption
module Sealed {
  public affine type Sealed[F,T](T)
  public sealFor[F,affine T](val:T) => Sealed(val)
  public unseal[F,affine T](capability:F, Sealed[F,T](val)) => val //only possesor of capability F can unseal
}
//Virtual Signature
module Authenticated {
  public transient affine type Signed[S,T](T)
  public sign[S,affine T](capability:S, val:T) => Signed[S,T](val) //only possesor of capability S can sign
}
```
