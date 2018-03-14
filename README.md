# Sanskrit
A Total smart contract virtual machine with affine type support.
This is the first part of my PhD project at the University of Zurich. 

## License
Copyright 2018 Markus Knecht, System Communication Group, University of Zurich.
Concrete Licence is not yet defined.

## Status
This is in a very early stage and currently just a description of what the goal is with no commited code yet.

## Why another smart contract virtual machine
Most currently used smart contract virtual machine couple the code and the state of a contract/blockchain object tightly. The code associated with a contract is the only code that can write and read to/from the associated storage/state. In exchange, this code can be granted arbitrary access to its storage and other local resources like its memory or stack and thus can be a simple low-level bytecode without a type system or any enforced guarantees. This model is simple and easy to implement but has some drawbacks as well. It is not possible to do cross contract optimisations like for example inlining a cross-contract call and further it is not possible to compile languages to it that require that certain guarantees given by the compiler hold at runtime as soon as these guarantees have to hold cross contracts. This prevents the use of alternative concepts and paradigms that may be a beneficial addition to smart contract programming. With Sanskrit VM and later a high-level language (codename: Mandala) such alternative concepts and paradigms will be explored.

## What is different about the Sanskrit virtual machine
The Sanskrit virtual machine does not only include a low-level bytecode interpreter but additionally a compiler that compiles a mid-level code representation into low-level bytecode. High-level languages do produce the mid-level code which then is compiled to low-level code. This compilation is part of the blockchain consensus and only low-level code produced by this compilation step is deployed to the blockchain. This allows having a type system as well as certain cross contract guarantees in the mid-level code that can not be circumvented at. Thanks to optimisations during the on chain compilation the runtime overhead can be kept near zero and often be eliminated completely. It does further allow to have optimisations like for example cross contract inlining. 

## What are the design decisions and features of the Sanskrit virtual machine

### Static Dispatch
The Sanskrit virtual machine does not have any dynamic function dispatches making reasoning about code easier. Tools and auditors can always be certain what code is executed on a call. This further allows aggressive inlining which reduces expensive disk reads needed for looking up the called functions code during execution and further reduces the number of necessary proofs if a stateless client model would be used. This model is, also, easier to be used in junction with formal verification compared to a model that includes dynamic dispatches.

### Total Language
The Sanskrit virtual machine is not Turing complete as it does not allow recursive function calls and does only allow loops with an upper bound of iterations. This does make it possible to calculate an upper bound on the resources consumed during a function call allowing to design alternative gas models that can never run out of gas by requiring the caller to have enough reserves to pay for the worst case execution path. 

### Algebraic Datatypes
The Sanskrit virtual machine is founded on immutable non-recursive algebraic datatypes as its fundamental representation of values giving it a functional touch with all the benefits coming from that. Sanskrit algebraic datatypes have some special properties that make them especially well suited for programming smart contracts in a way different from current approaches and idiomatic to Sanskrit.

#### Authentic Opaque Types
The Sanskrit types have by default two fundamental properties differentiating them from classical algebraic datatypes. First, only code belonging to the same Module (Sanskrit deployment unit) as the type can create values of that type (Authentic) and access the fields inside the type (Opaque). This ensures a holder of a value that it was constructed under certain circumstances dictated by the Module containing the type. If it is wished that a field of a type can be read by other Modules, the type can be declared Transparent. Further, a type can be declared Sealed, allowing other Modules to create instances. A type declared Open allows other modules to read fields and create new instances.

#### Affine Types
Besides varying in who can create and read a value, a Sanskrit type can additionally be declared to be Affine, and in that case, the compiler does enforce that a value of this type once created cannot be duplicated. Meaning that a function receiving a value of an affine type can use this value at most once. This makes affine types the perfect candidate for representing assets, tokens, cryptocurrencies etc... and thus the Sanskrit virtual machine does not have a native cryptocurrency that it must treat differently as they can conveniently be represented with the existing concepts.

### Generics
The Sanskrit virtual machine does support Generic Functions and Types meaning that a Type or Function can take another type as a parameter and thus can be defined in a more general type independent way. To interact with affine types, generic parameters on functions can be declared affine allowing the caller to instantiate them with an affine type. A generic algebraic datatype becomes affine if one of its generic parameters is instantiated with an affine type.

### Capabilities
The type system of Sanskrit is powerful enough to provide a capability-based access control system that has near zero runtime overhead and allows to check access control during compilation and thus code accessing values or calling functions it is not allowed to does not compile.

### Effect System
Sanskrit virtual machine makes a difference between three kinds of functions. Pure (default), plain, dependent and active functions. Pure functions can not create, read or write cells. Plain functions are like pure ones but can create new cells. Dependent function can in addition to plain functions read cells, and active functions have no limitations. This gives an easy way to detect which functions can be computed off-chain (pure, plain, dependent) and which need the state during the off-chain computation (dependent), as well as provides some potential for optimisations and making the job of auditors and static analysis tools simpler.

### Cells and References
Cells and References are Sanskrit way of providing persisted and shared state. A type can have a single assosiated initialisation function that can be used to generate a cell containing an initial value of that type. The creator does receive a reference to the cell. The same modifiers (Opaque, Transparent, Sealed and Open) define who can access the cell in what way. If a Module can create new instances of the value in the cell it can write to the cell and if it can access fields of the value it can read from the cell. Modification to the cell is represented as a pure state transition from the old to the new value and thus is not allowed to access other cells in the process (preventing shared state problems like reentrancy attacks).

#### View Types
View types are types that allow to generate references of a different type to the same cell from an existing reference to it. They are predestined to be used as cababilities that define how the posessor of the reference can interact with the cell and thus enable to program by the principle of Least Authority. A View type can not have an initialisation function itself and is restricted to a single constructor with a single field. If a cell is read over a view, then the returned value is imideately wrapped into a value of the view type and if a value of a view type is stored into a cell the inner value is extracted from the view type before storing it. An initialisation fucntion for a non-view type can return a value wrapped into a view type instead of the true value. Who can read and write to a cell is still governed by the true type of the cell but the modifiers (Opaque, Transparent, Sealed and Open) of the view type define who can wrap a reference into a view (needs create allowance) and who can remove a view wrapper (needs field access allowance). It is possible to apply multiple views to a reference and not just one. 

### Deterministic Parameterized Constants
Deterministic Parameterized Constants provide root storage slots in the Sanskrit virtual machine. They are very similar to functions but with one crucial difference. They behave as if the result is memorized, meaning if the constant is invoked with the same generic and value parameters multiple time then the same result is produced, this includes newly created references, meaning they will point to the same cell. Such a const must be pure or plain which ensures that the value of a const for a specific set of arguments is not dependent on when it is executed. This allows recomputing consts each time they are needed instead of storing its result (storage on a blockchain is usually way more expensive than calculations).
Without these consts, cells could not be used to persist state as every reference would only exist during one transaction and the next would create a new reference to a new cell even when obtained by calling the same function with the same arguments as the previous transaction did.

### Transactional
The Sanskrit virtual machine supports transactional functions in addition to normal ones. A transactional function can either return a result (if it is committed) or return the inputs (if it did a rollback). It is important that on a rollback the function arguments are returned as otherwise affine values could get lost. On a rollback, all modifications to cells are reverted to the value they had before the function call. A transactional function can call another transactional function by using the currently active transaction or by opening a new subtransaction. The second one is the only option for non-transactional functions calling a transactional one. When opening a subtransaction, the caller has to handle the commit and the rollback case. This can be implemented very efficiently in Sanskrit as it is well suited for how the interpreter currently is structured. Transactions and rollback can be used as an efficient error handling method with the limitation that they do not allow to communicate what went wrong but it gives a potentially high-level language the necessary tools to provide error handling mechanisms in the presence of affine types.

### Sharding Aware
The Sanskrit virtual machine is designed in a way that has certain benefits in a sharded environment. This includes that Module code is immutable and stateless and once the code is compiled and deployed it will always be there and has the same functionality independent of the active shard. This means that one shard could use code deployed on another shard or a dedicated deployment shard/chain. This makes it possible that values can be transferred between shards as the code that operates on them is available on other shards. Even references can be transferred between shards on each shard the reference initially points to a cell with the value containing the initial value (lazy initialised).

### Stateless Client Aware
The Sanskrit virtual machine is designed in a way that reduces the proof overhead in a stateless client model. Beside the already mentioned inlining references to other components that are not eliminated are represented as hashes of the target's content. This means that a proof of the entry point of the transaction is sufficient to proof the existence and validity of all code that can be executed during a function call. Further Sanskrit optimisations and overall paradigm encourage a programming style where the state is manipulated over pure functions and only persisted into cells when needed. It Further allows representing a lot of concepts like for example access control as pure type system concepts which do not need to access cells at all during runtime. If multiple cells are accessed, the proof may be slightly bigger in contrast to other smart contract virtual machines as related cells are not bundled under a common prefix (contract).

### Embeddable
The Sanskrit virtual machine is designed in a way that it can run beside another virtual machine that uses an account model like for example the Ethereum virtual machine and that it would be a comparatively low effort to allow the Host virtual machine to call into the Sanskrit virtual machine. The other way around is not as simple and would need further investigations. The compiler and interpreter parts will have some attention on performance with the goal that they eventually later may be run as smart contract on top of an existing smart contract virtual machine or at least with trusted computation services like TrueBit.

## Example Pseudo Code
Sanskrit requires a different programming style than other smart contract systems the following pseudocode should give a feel for what Sanskrit can do. 
Most of the presented code probably would be in a standard library. The syntax is just descriptional as real Sanskrit is a bytecode format.
The used syntax is inspired by the vision for the future high-level language Mandala that will compile to Sanskrit bytecode.
As Error handling in Sanskrit can get verbose very fast without the support of a higher level language a concept similar to the "?" from Rust was used here.

### Token
These Modules represent a Generic Token and related concepts. 

```
module Token {
  //T is the concrete token type as well as the minting capability
  public affine transient type Token[T](Int)                          
  //affine T means T can be instantiated by an affine type
  //only the possesor of a value of T can mint
  public mint[affine T](capability:T, amount:Int) => Token[T](amount) 
  
  //"?" Syntactic sugar for Result handling (similar to rusts ? but returning unconsumed affine inputs to preserve them)
  //"?" is used here to handle arithmetic errors during addition 
  public merge[affine T](Token[T](amount1), Token[T](amount2)) => Token[T](add(amount1,amount2)?)                                       
  public split[affine T](Token[T](amount), split:Int) => case sub(amount,split)? of 
                                                            (rem,split) => (Token[T](rem), Token[T](split)) 
  public zero[affine T]() => Token[T](0)
  .... //Probably more stuff
}

module Purse {
  public affine type Purse[T]
  //Capability allowing to withdraw funds (In reality would use Cap[Owner,Purse[T]] see later)
  public transient view type Owned[T](Purse[T])   
  //The creator is the Owner
  public init Purse[T] => Owned[T](Purse[T](Token.zero()))  

  public active deposit[affine T](purse: ref Purse[T], deposit:Token[T]) => modify purse with 
                                                                                   Purse(t) => Purse(Token.merge(t, deposit)?)
  public active withdraw[affine T](purse: ref Owned[T], amount:Int) => modify purse with 
                                                                        Owned(Purse(t)) => case Token.split(t,amount)? of
                                                                          (rem,split) => Owned(Purse(re)) &return split
}

module DefaultPurseStore{
  //maps each T address pair to a reference to a Owned Purse
  private const purse[T](address:Address) => new[Purse.Owned[T]]
  //"this" is the transaction initiator
  public getMyPurse[affine T]() => purse[T](this)  
  //unwrap removes the Owned view                                 
  public getPurse[affine T](address:Address) => purse[T](address).unwrap 
  public active transfer[affine T](trg:Address, amount:Int) => getPurse[T](trg).deposit(Purse[T].withdraw(getMyPurse[T], amount)?)? 
  
  .... //Probably some more stuff
}

```

#### Token Instantiation
```
import Token;     
module MyFixSupplyToken {
    //Identifier for Specific Token as well as Minting capability
    public type MyToken
    //MyToken instance is capability allowing to create Token[MyToken]
    on deploy => DefaultPurseStore.getMyPurse().deposit(Token.mint[MyToken](MyToken, 100000000))? 
}
```
### Generic Capabilities
```
module Capability {
  public transient view type Cap[C,T](T)
  public addCap[affine C,affine T](capability:C, value:ref T) => value.wrap[Cap[C,T]]
  //the pattern for the function parameters do an implicit unwrap of the view
  public combineCap[affine C1,affine C2,affine T](Cap[C1,T](ref val), Cap[C2,T](ref val2)) => case val == val2 of
                                                                                                  True => Some(val.wrap[Cap[(C1,C2),T]])
                                                                                                  False => None
  public splitCap[affine C1,affine C2,affine T](Cap[(C1,C2),T](ref val)) => (val.wrap[Cap[C1,T]], val.wrap[Cap[C2,T]])
  
  .... //probably much more
}

```

### Some Virtual Crypto
```
//Virtual encryption
module Sealed {
  public sealed type Sealed[F,T](T)
  //only possesor of capability F can unseal
  public unseal[affine F,affine T](capability:F, Sealed[F,T](val)) => val 
}

//Virtual Signature
module Authenticated {
  public transient type Signed[S,T](T)
  //only possesor of capability S can sign
  public sign[affine S,affine T](capability:S, val:T) => Signed[S,T](val)
}

//Virtual Threshold encription 
module Tresor {
  //Int -> needed Keys, Id -> special unique identifier type
  public affine type Tresor[affine T](Int,Id,T) 
  public affine type Keys(Int,Id)
 
  public create(total:Int, needed:Int, val:T) => let id = new Id in 
                            (Tresor[T](needed,id, val), Keys(total,id))
  
  public split(Keys(amount,id), split:Int) => case sub(amount,split)? of 
                            (rem,split) => (Keys(rem,id), Keys(split,id)) 
  
  public merge(Keys(amount1,id1), Keys(amount2,id2)) => case id1 == id2 of
                                True => Success(Keys(add(amount1,amount2)?,id1))                   
                                False => Failure((Keys(amount1,id1), Keys(amount2,id2)))
                                                            
  public open[T](Tresor[T](needed,id1,value), Keys(provided,id2)) =>  case id1 == id2 && needed <= provided of
                                      True => Success(value) 
                                    False => Failure((Tresor[T](needed,id1,value), Keys(provided,id2)))
                                                            
}

```
