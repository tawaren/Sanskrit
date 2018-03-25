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
The Sanskrit virtual machine does not only include a low-level bytecode interpreter but additionally a compiler that compiles a mid-level code representation into low-level bytecode. High-level languages do produce the mid-level code which then is compiled to low-level code. This mid to low-level compilation is part of the blockchain consensus and only low-level code produced by this compilation step is deployed to the blockchain. This allows having a type system as well as certain cross contract guarantees in the mid-level code that can not be circumvented by any high-level language. Thanks to optimisations during the on-chain compilation the runtime overhead to ensure such guarantees can be kept near zero and often be eliminated completely. It does further allow to have optimisations like for example cross contract inlining that would not be possible otherwise. 

## What is the goal of the Sanskrit virtual machine
The goal of the Sanskrit virtual machine is to explore new concepts and paradigms that are rarely if at all used in other language and evaluate if they provide a benefit for smart contract programming. The assumption behind this approach is that the smart contract programming environment is different enough to the classical (non-smart contract) programming enviroments such that it is plausible that approaches that are inappropriate in the later may be viable and benefical in the former.

## What are the design decisions and features of the Sanskrit virtual machine

### Static Dispatch
The Sanskrit virtual machine does not have any dynamic function dispatches making reasoning about code easier. Tools and auditors can always be certain what code is executed on a call. This further allows aggressive inlining which reduces expensive disk reads needed for looking up the called functions code during execution and further reduces the number of necessary proofs if a stateless client model would be used. This model is, also, easier to be used in junction with formal verification compared to a model that includes dynamic dispatches.

### Total Language
The Sanskrit virtual machine is not Turing complete as it does not allow recursive function calls and does only allow loops with an upper bound of iterations. Together with the absence of dynamic dispatches this does make it possible to calculate an upper bound on the resources consumed during a function call allowing to design alternative gas models that can never run out of gas by requiring the caller to have enough reserves to pay for the worst case execution path. 

### Algebraic Datatypes
The Sanskrit virtual machine is founded on immutable non-recursive algebraic datatypes as its fundamental representation of values giving it a functional touch with all the benefits coming from that. Sanskrit algebraic datatypes have some special properties that make them especially well suited for programming smart contracts in a way different from current approaches and idiomatic to Sanskrit.

#### Resticted Types
The Sanskrit types do by restrict the interaction possibilitties for functions considerably. By default functions cannot create values, access fields, copy values, discard values, wrap references, unwrap references or persist values (see cells and views for the last three). When declaring a type, these restriction can individually be removed to create a type that provides the needed behaviour. Some of the powers (all except read, create, wrap and unwrap) are recursive, meaning that an algebraic data type can only have these powers if the parameters of all constructor fields have them as well. Functions that are defined in the same Module as the type are always treated as if they have the non-recursive powers even if the type declaration does not grant these powers.
For Example a type with access, discard and perstsit powers make the perfect candidate for representing assets, tokens, cryptocurrencies etc... and thus the Sanskrit virtual machine does not have a native cryptocurrency that it must treat differently as they can conveniently be represented with the existing concepts. 

### Generics
The Sanskrit virtual machine does support generic functions and types meaning that a type or function can take other types as parameters and thus can be defined in a more type independent way. To interact with the restriction system, generic parameters on functions must declare if they support additional powers (lifted restrictions) preventing the caller to instantiate them with a type that does not have the requested power. If a type parameter in an algebraic datatype is less powerfull in respect to its recursive powers (all except access and create) then the resulting types powers are reduced accordingly. A type parameter can be marked phantom and in that case it can not be used as a type for a constructor parameter and can only used as generic parameter to other phantom type parameters, in return phantom generic parameters never strip power away from the resulting type.

### Capabilities
The type system of Sanskrit is powerful enough to provide a capability-based access control system that has near zero runtime overhead and allows to check access control during compilation and thus code accessing values or calling functions it is not allowed to does not compile.

### Cells and References
Cells and References are Sanskrit way of providing persisted and shared state. Beside normal data types, their are cell types that can be used as ordinary data types but have an assosiated initialisation function that can be used to generate a cell containing an initial value of that type. The creator does receive a reference to the cell instead of the blank value. The powers regulate who can access the cell in what way. If a Module can create new the value of the cells type then it can write to the cell and if it can access fields of the value it can access the cell. Certain usecases need types that can never be persisted, to represent this the new persist power is needed to create a cell of a type. The persisted power is unlike the create power a recursive power. Modifications to a cell are represented as a pure (side effect free) state transition from the old to the new value and thus are not allowed to access other cells in the process (preventing shared state problems like reentrancy attacks).

#### View Types
View types are types that allow to generate references of a different type to the same cell from an existing reference to it. They are predestined to be used as capabilities that define how the posessor of the reference can interact with the cell and thus enable to program by the principle of Least Authority. A View type can not have an initialisation function itself and is restricted to a single constructor with a single field. If a cell is accessed over a view, then the returned value is imideately wrapped into a value of the view type and if a value of a view type is stored into a cell the inner value is extracted from the view type before storing it. An initialisation fucntion for a non-view type can return a value wrapped into a view type instead of the true value. Who can read and write to a cell is governed by the power of type of the cell (read and create are treated as recursive in the context of views) but the view type declares seperately who can wrap a reference into a view  and who can remove a view wrapper (two new non-recursive powers only usable with views). It is possible to apply multiple views to a reference and not just one. 

### Effect System
Sanskrit virtual machine makes a difference between four kind of functions. Pure (default), plain, dependent and active functions. Pure functions can not create, access or write cells. Plain functions are like pure ones but can create new cells. Dependent function can in addition to plain functions access cells, and active functions have no limitations. This gives an easy way to detect which functions can be computed off-chain (pure, plain, dependent) and which need the state during the off-chain computation (dependent), as well as provides some optimisation potential for non-active functions. Additionally this does make the job of auditors and static analysis tools simpler.

### Deterministic Parameterized Constants
Deterministic Parameterized Constants provide root storage slots in the Sanskrit virtual machine. They are very similar to functions but with a crucial difference. They behave as if the result is memorized, meaning if the constant is invoked with the same generic and value parameters multiple time then the same result is produced, this includes newly created references, meaning they will point to the same cell. Such a const must be a pure or plain function and return a type with the store and copy restrictions lifted to ensures that the value of a const for a specific set of arguments is not dependent on when and how often it is created. This allows recomputing consts each time they are needed instead of storing its result (storage on a blockchain is usually way more expensive than calculations).
Without these consts, cells could not be used to persist state as every reference would only exist during one transaction and the next would create a new reference to a new cell even when obtained by calling the same function with the same arguments as the previous transaction did.

### Transactional
The Sanskrit virtual machine supports transactional functions in addition to normal ones. A transactional function can either return a result (if it did a committ) or return the inputs and an error code (if it did a rollback). It is important that on a rollback the function arguments are returned as otherwise values could get lost. On a rollback, all modifications to cells are reverted to the value they had before the function call. A transactional function can call another transactional function by using the currently active transaction or by opening a new subtransaction. The second one is the only option for non-transactional functions calling a transactional one. When opening a subtransaction, the caller has to handle the commit and the rollback case. This can be implemented very efficiently in Sanskrit as it is well suited for how the interpreter currently is structured. Transactions and rollback can be used as an efficient error handling method with the limitation that they do only allow to communicate an error in case of a rollback and not a detailed description. This gives the necessary tools to a high-level language for provide error handling mechanisms in the presence of restricted types.

### Sharding Aware
The Sanskrit virtual machine is designed in a way that has certain benefits in a sharded environment. This includes that Module code is immutable and stateless and once the code is compiled and deployed it will always be there and has the same functionality independent of the active shard. This means that one shard could use code deployed on another shard or a dedicated deployment shard/chain. This makes it possible that values can be transferred between shards as the code that operates on them is available on other shards. Even references can be transferred between shards on each shard the reference initially points to a cell with the value containing the initial value (lazy initialised).

### Stateless Client Aware
The Sanskrit virtual machine is designed in a way that reduces the proof overhead in a stateless client model. Beside the already mentioned inlining references to other components that are not eliminated are represented as hashes of the target's content. This means that a proof of the entry point of the transaction is sufficient to proof the existence and validity of all code that can be executed during a function call. Further Sanskrit optimisations and its overall paradigm encourage a programming style where the state is manipulated over pure functions and only persisted into cells when needed. It Further allows representing a lot of concepts like for example access control as pure type system concepts which do not need to access cells at all during runtime. If multiple cells are accessed, the proof may be slightly bigger in contrast to other smart contract virtual machines as related cells are not bundled under a common prefix (contract address).

### Embeddable
The Sanskrit virtual machine is designed in a way that it can run beside another virtual machine that uses an account model like for example the Ethereum virtual machine and that it would be a comparatively low effort to allow the host virtual machine to call into the Sanskrit virtual machine. The other way around is not as simple and would need further investigations. The compiler and interpreter parts will have some attention on performance with the goal that they eventually later may be run as smart contract on top of an existing smart contract virtual machine or at least with the help of trusted computation services like TrueBit.

## Example Pseudo Code
Sanskrit requires a different programming style than other smart contract systems the following pseudocode should give a feel for what Sanskrit can do and how it achieves it. Most of the presented code probably would be in a standard library. The syntax is just descriptional as real Sanskrit is a bytecode format. The used syntax is inspired by the vision for the future high-level language Mandala that will compile to Sanskrit bytecode.
As Error handling in Sanskrit can get verbose very fast without the support of a higher level language a concept similar to the "?" from Rust was used here to make the code more readable.

### Token

```
module Token {
  //T is the concrete token type as well as the minting capability
  powers<discard, access, persist>
  public data type Token[phantom T](Int)                          
  //only the possesor of a value of T can mint
  public mint[discard T](capability:T, amount:Int) => Token[T](amount) 
  
  //"?" Syntactic sugar for Result handling (similar to rusts ? but returning unconsumed affine inputs on a failure to preserve them)
  //"?" is used here to handle arithmetic overflows during addition 
  public merge[T](Token[T](amount1), Token[T](amount2)) => Token[T](add(amount1,amount2)?)                                       
  public split[T](Token[T](amount), split:Int) => (Token[T](sub(amount,split)?), Token[T](split)) 
  public zero[T]() => Token[T](0)
  .... //Probably more stuff
}

module Purse {
  powers<discard, access, persist>
  public cell type Purse[phantom T](Token[T])
  //Capability allowing to withdraw funds 
  powers<discard, access, persist, unwrap>  //unwrap is the power to remove the view
  public view type Owned[phantom T](Purse[T])   
  //The creator recieves a reference with the Owned view, which represents the withdraw capability
  public init Purse[T] => Owned[T](Purse[T](Token.zero()))  

  public active deposit[T](purse: ref Purse[T], deposit:Token[T]) => modify purse with 
                                                                            Purse(t) => Purse(Token.merge(t, deposit)?)
  public active withdraw[T](purse: ref Owned[T], amount:Int) => modify purse with 
                                                                       Owned(Purse(t)) => case Token.split(t,amount)? of
                                                                            (rem,split) => Owned(Purse(rem)) &return split
}

module DefaultPurseStore{
  //maps each T, address pair to a reference to a Owned Purse
  private const purse[T](address:Address) => new[Purse.Owned[T]]
  //"self" is the transaction initiator
  public plain getMyPurse[T]() => purse[T](self)  
  //unwrap removes the Owned view                                 
  public plain getPurse[T](address:Address) => purse[T](address).unwrap 
  public active transfer[T](target:Address, amount:Int) => Purse.deposite(getPurse[T](target),Purse.withdraw(getMyPurse[T], amount)?)?
  .... //Probably some more stuff
}

```

#### Token Instantiation
```
module MyFixSupplyToken {
    //Identifier for Specific Token as well as Minting capability
    powers<discard>
    public data type MyToken
    //MyToken instance is used as capability allowing to create Token[MyToken]
    on deploy => DefaultPurseStore.getMyPurse().deposit(Token.mint[MyToken](MyToken, 100000000))? 
}
```
### Some Virtual Crypto
```
//Virtual encryption
module Sealed {
  //All except access
  powers<create, discard, copy, persist> // T may strip some powers away  
  public data type Sealed[phantom F,T](T)
  //only possesor of capability F can unseal
  public unseal[discard F,T](capability:F, Sealed[F,T](val)) => val 

}

//Virtual Signature
module Authenticated {
  //All except create
  powers<access, discard, copy, persist> // T may strip some powers away 
  public data type Signed[phantom S,T](T)
  //only possesor of capability S can sign
  public sign[discard S,T](capability:S, val:T) => Signed[S,T](val)
}

//Virtual Threshold encription 
module Tresor {
  //Int -> needed Keys, Id -> special unique identifier type
  //All except read and create
  powers<discard, copy, persist> // T may strip some powers away 
  public data type Tresor[T](Int,Id,T) 
  powers<read,discard,store>
  public data type Keys(Int,Id)
 
  public create[T](total:Int, needed:Int, val:T) => let id = new Id in (Tresor[T](needed,id, val), Keys(total,id))
  
  public split(Keys(amount,id), split:Int) => (Keys(sub(amount,split)?,id), Keys(split,id)) 
  
  public merge(Keys(amount1,id1), Keys(amount2,id2)) => Keys(add(amount1,amount2)?,assert(id1==id2,id1)?)                               
  public open[T](Tresor[T](needed,id1,value), Keys(provided,id2)) => assert(id1 == id2 && needed <= provided, value)

                                                            
}

```
