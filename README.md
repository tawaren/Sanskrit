

# Sanskrit
A smart contract execution and deployment platform with opaque and affine type support.    
This is part of my PhD thesis at the University of Zurich.

## License

Copyright (C) 2024 Markus Knecht, System Communication Group, University of Zurich.

This project is licensed under the GNU General Public License v3.0. You can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.

## Status

**Research Grade Software**

This software is currently in a research-grade state and is not production-ready. It has been developed for testing and evaluation purposes.   While it includes all necessary components for these purposes, additional work is required to make it suitable for production environments.   Especially, proper resource metering and fee payment mechanisms are not fully implemented.

**Use at Your Own Risk**

Use of this software is at your own risk.  The developers and contributors are not responsible for any damage or loss that may result from using this software.  It is provided "as-is" without any warranties or guarantees.

## Why another platform to execute smart contracts
Most currently used smart contract execution platforms couple the code and the state of a contract/blockchain object tightly.   The code associated with a contract is the only code that can write and read to/from the associated storage/state.   In exchange, this code can be granted arbitrary access to its storage and other local resources like its memory or stack and thus can be a simple low-level bytecode without a type system or any enforced guarantees.

This model is simple and easy to implement but has some drawbacks as well.  It is not possible to do cross contract optimisations like for example inlining a cross-contract call and further it is not possible to compile languages to it that require that global guarantees given by the compiler hold at runtime.   This prevents the use of alternative concepts, features and programming styles that may be a beneficial addition to smart contract programming.

With Sanskrit and a high-level language, like Mandala (integrated into the [Samaya](https://github.com/tawaren/Samaya) build tool), such alternative concepts and paradigms can be exlored.

## What is different about the Sanskrit platform
The Sanskrit  platform does not only consist of a low-level bytecode interpreter but additionally a code verifier and transpiler to check and traspile a mid-level code representation into low-level bytecode during code deployment.     
High-level languages produce the mid-level code which then is traspiled to low-level code.     
This verifiaction and mid to low-level transpilation is part of the blockchain consensus and only low-level code produced by this step is deployed to the blockchain.     
This allows having a type system as well as certain cross contract guarantees in the mid-level code that can not be circumvented by any high-level language.     
Thanks to optimisations during the on-chain compilation the runtime overhead to ensure such guarantees can be kept near zero and often be eliminated completely.

## What is the goal of the Sanskrit platform
The goal of the Sanskrit platform is to explore new concepts and paradigms that are rarely if at all used in other language, especially smart contract languages, and evaluate if they provide a benefit for smart contract programming.   The assumption behind this approach is that the smart contract programming environment is different enough to the classical (non-smart contract) programming enviroments such that it is plausible that approaches that are inappropriate in the later may be viable and benefical in the former.

## What are the design decisions and features of the Sanskrit platform
These section shows some of the outstanding key features of Sanskrit but is not complete and ommits many aspects. For a more indept exploration of the features see the corresponding PhD thesis (the link follows after puplication).

### Functions
#### Pure Functions
The Sanskrit byte-code language is purely functional in the regard that it only allows for pure side effect free functions. Unlike in other smart contract platforms the state of an smart contract is always passed in as parameter to a function which can return an updated state. Transactions specify which smart contract state to load from the blockchains state and pass in as parameter to a function. The advanced type system will guarantee that proper state isolation is maintained.

#### Static Dispatch
The Sanskrit platform does primarely us static function dispatches making reasoning about code easier.  The only exceptions are signatures and implements intended to implement a limited form of higher order functions or type class like features.

This allow tools and auditors to be certain what code is executed on a call. This further prevents expensive disk reads needed for looking up the called functions code during execution as they can already be collected during transpilation. This model is, also, easier to be used in junction with formal verification compared to a model that primarely uses dynamic dispatches like most other smart contracts do.

### Ressource Bounded Language
The Sanskrit byte-code programming language is not Turing complete as it does not allow recursive function calls or loops.  Together with the limitation placed on dynamic dispatches this make it possible to calculate an upper bound on the resources consumed during a function call allowing to design alternative gas models that can never run out of gas by requiring the caller to have enough reserves to pay for the worst case execution path.

### Data Types
The Sanskrit platform uses immutable non-recursive algebraic datatypes as its fundamental representation of values giving it a functional touch with a lot of the benefits coming from that.  Sanskrit algebraic datatypes have some special properties that make them especially well suited for programming smart contracts in a way different from current approaches and idiomatic to Sanskrit.

#### Opaque and Substructural Types
The Sanskrit types do by default restrict the interaction possibilitties for functions with values of that type considerably.   By default functions cannot create, copy, deconstruct, or discard values.  When declaring a type, these restriction can individually and selectively be lifted, by granting  priviledges to code, resulting in a finetuned type that provides the needed behaviour.     
Some of the privileges (like copy and discard) are recursive, meaning that an algebraic data type can only have these privileges if the parameters of all constructor fields have them as well.

Functions that are defined in the same Module (deployment unit) as the type are always treated as if they have all the non-recursive privileges even if the type declaration does not grant these privileges.  By not allowing a value to be copied and/or discarded so called substructural types can be created.  For Example a type without copy priviliges make the perfect candidate for representing assets, tokens, cryptocurrencies etc... and thus the Sanskrit platform does not have a native cryptocurrency that it must treat differently as they can conveniently be represented with the existing concepts.

### Type Parameters
The Sanskrit platform does support generic functions and types meaning that a type or function can take other types as parameters and thus can be defined in a reusable way.   To integrate with the type system, type parameters on functions must declare if they require additional priviledges preventing the caller to instantiate them with a type that does not grant the requested privileges.  If a type parameter in an algebraic datatype is less priviledged then the data type itself in respect to its recursive priviledges then the applied types priviledges are reduced accordingly.

### Capabilities
The type system of Sanskrit is powerful enough to provide a capability-based access control system that has near zero runtime overhead and check many access control related aspects during compilation and code violating access does often not even compile.

These is achieved in two ways: First as values in Sanskrit can be scarce (using substructural types) they can be used as access token. This kind of capability is called a dynamic capability.  Second a function can be declared as protected by one of its generic arguments and then can only be called by code from a module that declares the type applied to that type parameter.  This is called a static capability.

### Error Handling
As Sanskrit uses functional concept errors can be returned as result of a function using algebraic data types like Option[T] or Either[T,E]. However, this is often inconvionient with substructural types as unused parameters may have to be returned as well on an error (if they can not be discarded). Thus Sanskrit has transactional functions that can fail. On a success they return the return values and on a fail the return the parameters. a call to such a function can takes two branches one for the success and one for the fail case, if they dont the calling function fails if the called function fails.

### Transaction Scripts
Sanskrit transaction do contain scripts, that load values from the blockchain states and then invokes functions with them and finnaly store the result back in to the blockchain states. This allows to invoke more than one function in a transaction. A transaction can even be signed by multiple parties and thus exchanges and other atomic opearations require a single transaction instead of multiple as it is often the case in other smart contract platforms.

## Example Mandala Code
Sanskrit requires a different programming style than other smart contract systems the following Mandala code should give a feel for what Sanskrit can do and how it achieves it.  Mandala is used, as Sanskrit byte code does not have a human readable representation. Only a single module is presented to get a feel for it, more examples are found in the Mandala standard library (LINK).


### Token
The token example ommits imports and some helper function as well as class instances, the complete code is in the standard library.

``` module Token {  
 // T is the type identifing the Token (phantom means it is not used as field param) // linear is defines a substructural type (the values can not be copied or discarded) // global(inspect) means everyone can read the amount // local(create, consume) means only code in this module can create and destroy values of this type global(inspect) local(create, consume) linear data Token[phantom T](amount:U128)    
   // function to mint new tokens of type T with the amount passed as parameter  
 // guarded[T] means that this function can only be called by the module defining T //  fot exmaple mint[Fix] can only be called by the module defining Fix guarded[T] function mint[T](amount:U128):(res:Token[T]) = Token#(amount)//global function zero[T]():(res:Token[T]) = Token#(0)   
 // function to allow definer of T to burn tokens of type T   guarded[T] function burn[T](consume tok1:Token[T]) = let Token(amount) = tok1 in ()    
    
   // function that splits a Token into two, preserving the total amount of tokens in the   
   // consume tok:Token[T] means, that this function gets the liear token (the caller looses access to it)  
 // transactional means that the function can fail (rolling it back) global transactional function split[T](consume tok:Token[T], split:U128):(reminder:Token[T], extracted:Token[T]){ // Aboort if not enough tokens are avaiable for the split ensure tok.amount >= split in        (Token#(tok.amount - split), Token#(split))    
   }    
     
   // function that merges two Tokens, preserving the total amount of tokens in the process   
   global transactional function merge[T](consume tok1:Token[T], consume tok2:Token[T]):(res:Token[T]) {    
        Token#(tok1.amount + tok2.amount)    
} }  
  
module FixToken {    
    //Marker type for our token    
    local data Fix  
 // Helper type to ensure it can only be initialized once    local linear data Inited()    
    
    // Defines who owns freshly minted tokens  
 global function initialOwner() = Subject#(idFromData(0xfe101df1f61a2facede83909190cffd71ab18e61)) // Defines how many token are minted initially (and totaly) global function totalSupply():U128 = 10000    
    // Function that initializes the token by minting totalSupply() for initialOwner()  
 //  The return values represent the initialisation state result //    Entry[Locked[Token[Fix]]] => A state stored in the Blockchain (Entry), that only can be accessed by a single subject (Locked) and contains a fix token (Token[Fix]) //    Entry[Inited] => A state that ensures that init can be called at most once. A second call would return a Entry with the same id, which would fail as the entry already exist, global transactional function init(consume gen:IdGenerator):(Entry[Locked[Token[Fix]]], Entry[Inited]) {        // Generate an identifier to store the minted tokens under  
 let (storageLoc, _) = uniqueID(gen) in        // Mint totalSupply tokens  
 let newTokens = mint[Fix](totalSupply()) in        // Lock the tokens such as only initialOwner can access them  
 let ownedTokens = lockEntry(initialOwner(),storageLoc,newTokens) in        // derive a deterministic id for this module  
 let initOnceId = privateModuleIdDerive(moduleId(), hash(0:U128)) in        // create an entry with the deterministic id so that multiple calls result in the same entry  
 let initOnceEntry = Entry#(initOnceId, Inited#()) in        // Return the tokens and the call once insurance (not as the insurance is linear it can not simply be discarded and must be stored)  
 (ownedTokens, initOnceEntry)} }  
  
// Exports the Token.init[Fix] function as top level function to be called from transactions  
transactional transaction InitFix(consume gen:IdGenerator):(coins:Entry[Locked[Token[Fix]]], blocker:Entry[Inited]) {    
init(gen) }  
  
// Exports a top level function to transfer init tokens  
// Parameters:  
//   coins:Entry[Locked[Token[Fix]]] => Tokens to spend from  
//   auth:Authorization => Authorisation to unlock the tokens  
//   amount:U128 => number of tokens to send  
//   to:Subject => receiver of the tokens  
// Returns:  
//   coinsChange:Entry[Locked[Token[Fix]]] => the not sendt tokens //   coinsSent:Entry[Locked[Token[Fix]]] => the sendt tokens  
transactional transaction TransferFix(consume coins:Entry[Locked[Token[Fix]]], auth:Authorization, amount:U128, to:Subject, context consume gen:IdGenerator):(coinsChange:Entry[Locked[Token[Fix]]], coinsSent:Entry[Locked[Token[Fix]]]) {   
  // Get access to the tokens using Authorization   
  let (senderLoc, from, allTokens) = unlockEntry(coins, auth) in    
  // Generate a new id for the sendt tokens  
 let (receiverLoc, _) = uniqueID(gen) in  // Split the token in two  
 let (rem,send) = split(allTokens, amount) in // Lock the remaining tokens again using same id and subject as the original let changeEntry = lockEntry(from,senderLoc,rem) in  // Lock the snedt tokens using new id and receiver subject  
 let receiverEntry = lockEntry(to,receiverLoc,send) in  // Return the tokens  
 (changeEntry, receiverEntry) }  
  
```   

#### Token Transactions
``` //Command to generate and execute a transaction calling InitFix and storing the result in the blockchains state  
exec InitFix(inject):(store(accA),store(initT))  
  
//Command to generate and execute a transaction calling that transfers 4000 bundle // Verifies the generated signature for A to generate an authorisation txt AuthorizationPermFromEdDsa(pk(A),sig(A),inject):(assign(auth)) // Uses the authorisation and coins generated by the previous transaction to send tokens to B  
//   Note: This will fail if account A is not 0xfe101df1f61a2facede83909190cffd71ab18e61  
txt TransferFix(consume(accA),read(auth),u128(4000),subject(B),inject):(store(accA),store(accB1)) exec  
  
```  

## Build Guide
The Sanskrit platforms end goal is to be used in a blockchain.  
It can be used as a dependecy in case the blockchain is written in Rust or it can be compiled to WASM and then be embedded.   However, while the components are implemented in a way so that they can compile to WASM, only the compilation and deployment create has WASM interfaces defined. Thus this build guide describes the build process to use it without WASM.   Further, this repository contains a local testing and evaluation server that provides a wallet and blockchain emulator.

As the software is research grade using it in a blockchain is only recommended if the target is research grade as well or this code is first made production ready.

### Blockchain Integration
As main orientation on how to interact with the Sanskrit platform use the sanskrit_local_server create as guide.  
The manager.rs is a good starting point, especially the impl State part which shows how to interact with the runtime.   
The externals module, respectively its sub modules show how to implement system types and functions.

The following crates are needed to use the Sanskrit platform:
- sanskrit_common: defines types and traits to interact with the platform
- sanskrit_runtime: defines the main entry points like deploy and execute
- sanskrit_compile: defines some types and traits used to implement the compilation of system functions
- sanskrit_interpreter: defines some types and traits used to implement the execution of system functions

The following crates can help but are not required:
- sanskrit_derive: If you want tho define your own types that use Sanskrit parsing and serialisation
- sanskrit_core: If you want to inspect deployed code, like Modules or Transaction Functions

The following tasks are essential for an integration:
- implement the sanskrit system modules
  - Additionally it is recomended to have a standard library
  - Note:  Their is a Standard Library including Systems module default options under [Mandala-Libs-And-Examples](https://github.com/tawaren/Mandala-Libs-And-Examples), written in Mandala (they provide the default Entry type).
- implement sanskrit_common::Store trait
  - this provides the functionality for Sanskrit to read and write to the Blockchains state
- implement sanskrit_runtime::direct_stored::SystemDataManager trait if you use the default Entry type
  - provides functions to identify the entry type (from module hash and type index)
  - provides functions to create special values provided by the Blockchain (like for example a block number)
  - provides function to get information about provided values like their size or gas cost to create them
- implement sanskrit_runtime::verify::TransactionVerificationContext trait if you use a custom state type
  - provides functions to read transaction descriptions from the Blockchain state
  - tracks the gas costs for reading and storing state
  - tracks the gas for creating values provided by the Blockchain (like for example a block number)
- implement sanskrit_runtime::verify::TransactionVerificationContext trait if you use a custom state type
  - provides functions to read transaction descriptions from the Blockchain state
  - provides functions to read, write and delete values to the Blockchain state
  - provides functions to create values provided by the Blockchain (like for example a block number)
- implement sanskrit_interpreter::externals::RuntimeExternals
  - provides functions to execute system functions (externals/pre compiles like hash functions)
- implement sanskrit_compile::externals::CompilationExternals
  - provides functions to compile system functions and data types (like an integer and addition)
  - Note: the sanskrit interpreter supports primitive types that can be used (see sanskrit_local_server::externals)
- implement sanskrit_runtime::Tracker trait
  - this provides callbacks about events happening during transaction script execution
  - this is optional and a implementation with all empty bodies is fine if no feedback is required
- implement sanskrit_runtime::system::SystemContext to wire everything together
  - Defines what to call for executing system functions (anskrit_interpreter::externals::RuntimeExternals implementation)
  - Defines how to store values into the blockchain state (sanskrit_common::Store implementation)
  - Defines the transaction type (sanskrit_runtime::TransactionBundle implementation - sanskrit_runtime::BundleWithHash can be used if no customization is needed)
  - Defines the context to use when verifying transactions (sanskrit_runtime::verify::TransactionVerificationContext implementation - sanskrit_runtime::direct_stored::StatefulEntryStoreVerifier can be used if no customization is needed)
  - Defines the context to use when executing transactions (sanskrit_runtime::compute::TransactionExecutionContext implementation - sanskrit_runtime::direct_stored::StatefulEntryStoreExecutor can be used if no customization is needed)

### Test and Evaluation Server

An implementation using the default entry type and and default externals corresponding to what is required by the system library in [Mandala-Libs-And-Examples](https://github.com/tawaren/Mandala-Libs-And-Examples). This implementations stores the state to the disk and generates one block per transaction. It just emulates the blockchain and is not a blockchain node. It has neiter a peer to peer network nor a consensus mechanism or other blockchain components. It provides just enough to test and evaulate the Sanskrit platfrom.

Use the following command to build the local server:  
``` cargo build --package sanskrit_local_server  --release```

This produces the executable in target/release/sanskrit_local_server  (the file has a platform specific ending like .exe for windows)

Starting the executable will listen to port 6000 for deployments. The [Samaya](https://github.com/tawaren/Samaya) build tool has a plugin that can deploy to this endpoint.  A db folder will be created to store the state in.

Further, a command line interface is provided to execute transactions. However, before anything usefull can be done at least a system library must be deployed, preferably a standard library as well. The [Samaya](https://github.com/tawaren/Samaya) build tool is caple of compiling and deploying the standard and system library at [Mandala-Libs-And-Examples](https://github.com/tawaren/Mandala-Libs-And-Examples).
The  [Mandala-Libs-And-Examples](https://github.com/tawaren/Mandala-Libs-And-Examples) repository further contains some examples that can be used to try it out (including commands for transactions that can be executed on the local servers)

For an indepth explanation of the command line interface consult the PhD thesis (the link follows after puplication).

To reset the state simply stop the local server and delete the db folder before starting it again.