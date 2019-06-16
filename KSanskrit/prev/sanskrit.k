module SANSKRIT-SYNTAX

  syntax Exps ::= List{Exp,","}
  syntax Exp ::= "Ret" "(" "[" OpCodes "]" "," "[" ValueRefs "]" "," "[" ValueRefs "]" ")"
              | "Throw" "(" ErrorPath ")"
  syntax OpCodes ::= List{OpCode,","}
  syntax ValueRefs ::= List{ValueRef,","}
  syntax ValueRef ::= Int

  //todo: strictly evaluate to base Path or even to Type
  syntax TypeDef ::= ModuleId "." TypeId "[" TypeDefs "]"                           [strict(3)]
                   | GenericId
  syntax TypeDefs ::= List{TypeDef,","}

  syntax FunctionDef ::= ModuleId "." FunctionId "[" TypeDefs "]"

  syntax ErrorPath ::= ModuleId "." ErrorId

  syntax ModuleId ::= Id | "native" | "this"
  syntax TypeId ::= Id | "u8" | "i8" | "bool" //todo: more later
  syntax ErrorId ::= Id | "NumericError"
  syntax FunctionId ::= Id

  syntax Lit ::= Int //for now later needs data as well
  syntax OpCode ::= "Lit" "(" Lit "," TypeDef ")"                                   [strict(2)]
                |   "Let" "(" Exp ")"                                               [strict]
                |   "Copy" "(" ValueRef ")"
                |   "Fetch" "(" ValueRef ")"
                |   "BorrowFetch" "(" ValueRef ")"
                |   "Free" "(" ValueRef ")"
                |   "Drop" "(" ValueRef ")"
                |   "BorrowUnpack" "(" ValueRef "," TypeDef ")"                     [strict(2)]
                |   "Unpack" "(" ValueRef "," TypeDef ")"                           [strict(2)]
                |   "BorrowSwitch" "(" ValueRef "," TypeDef "," "[" Exps "]" ")"    [strict(2)]
                |   "Switch" "(" ValueRef "," TypeDef "," "[" Exps "]" ")"          [strict(2)]
                |   "BorrowPack" "(" TypeDef "," TagId "," "[" ValueRefs "]" ")"    [strict(1)]
                |   "Pack" "(" TypeDef "," TagId "," "[" ValueRefs "]" ")"          [strict(1)]
                |   "Invoke" "(" FunctionDef "," "[" ValueRefs "]" ")"
                |   "Try" "(" Exp "," "[" Cases "]" ")"

  syntax TagId ::= Id
  syntax Cases ::= List{Case,","}
  syntax Case  ::= "(" ErrorPath "," Exp ")"

  //Todo: maybe other file later
  syntax Visibility ::= "private"
  syntax GenericId ::= Int

endmodule

module SANSKRIT-CONFIG
  imports SANSKRIT-SYNTAX

  syntax Id ::= "none"
  syntax Exp ::= "empty"

  syntax Loc ::= Id      //will be generated with !L:Id //unique new var

  //Todo: maybe other file later
  syntax Generic ::= generic(Set,Bool)   //the Id is generated (unique ident)

  configuration <T>
                    <k> $Exp:Exp </k>
                    <curModule> none </curModule>
                    <tenv> .List </tenv>       //Int => Type //todo: a list would be more appropriate analog to stack??
                    //<activeRisks> .Set </activeRisks>
                    <frames> .List </frames>    //List of Ints (start Indexes on the Stack
                    <stack> .List </stack>      //List of Locations
                    <store> .Map  </store>      //Location -> Elem
                    <modules>
                      <mod multiplicity="*">
                          <modName> none </modName>
                          <types>
                              <type multiplicity="*">
                                  <typeName> none </typeName>
                                  <typeGenerics> .List </typeGenerics>
                                  <typeCaps> .Set </typeCaps>
                                  <ctrs>
                                    <ctr multiplicity="*">
                                      <ctrName> none </ctrName>
                                      <ctrParams> .List </ctrParams>
                                    </ctr>
                                  </ctrs>
                              </type>
                          </types>

                          <errors>
                              <error multiplicity="*">
                                  <errorName> none </errorName>
                              </error>
                          </errors>

                          <funs>
                              <fun multiplicity="*">
                                  <funName> none </funName>
                                  <funGenerics> .List </funGenerics>
                                  <funVisibility> private </funVisibility>
                                  <funRisks> .Set </funRisks>
                                  <funParams> .List </funParams>
                                  <funReturns> .List </funReturns>
                                  <funBody> empty </funBody>
                              </fun>
                          </funs>
                      </mod>
                    </modules>
                </T>
endmodule


module SANSKRIT-SEMANTIC
    imports SANSKRIT-SYNTAX
    imports SANSKRIT-CONFIG

    syntax Exp ::= Result
    syntax Result ::= return(List)      //List of elements
                   |  exception(Error)
                   |  processReturns(ValueRefs,ValueRefs) [function]

    syntax Elem ::= elem(Value,Type,Int,Set)    //Int is locks, Set is Borrows

    syntax KResult ::= Result | Elem | Type

    syntax Value ::= Leaf | Node
    syntax Leaf ::= Int
    syntax Node ::= node(TagId,List)              //Int = Tag, List of Elems

    syntax Type ::= type(Set,BaseType)          //Set of Capabilities
                 | Generic
    syntax Types ::= List
                  | List{Type, ","}

    syntax TypeDef ::= Type | BaseType
    syntax TypeDefs ::= Types

    syntax List ::= toList(Types)    [function]

    syntax BaseType ::= base(ModuleId, TypeId, Types)
                      | var(Id)                       //id is generated (it is a type var)
                      

    syntax Cap ::= RecCap | NonRecCap
    syntax RecCap ::= "Drop" | "Copy" | "Persist"
    syntax NonRecCap ::= NormCap | SpecialCap
    syntax NormCap ::= "Embed" | "Create" | "Consume" | "Inspect"
    syntax SpecialCap ::= "Indexed"

    syntax Set ::= intersectCaps(Set,  List)    [function]    //Later is List of Types
               | transferNonRecCaps(Set, Set)   [function]

    syntax List ::= checkAndFilterApplies(List, Types) [function]
    //Type Resolution rules
   
    rule <k> GId:Int => TEnv[GId] ... </k>
         <tenv> TEnv:List </tenv>
    
    rule (M:ModuleId) . T:TypeId [ TS:Types ] => base(M, T, toList(TS))
    rule toList((T:Type,TS:Types)) => ListItem(T) toList(TS)

    rule <k> base(MId:Id, TId:Id, TL:List) => type(transferNonRecCaps(SCS,intersectCaps(SCS,checkAndFilterApplies(LS, TL))),base(MId, TId, TL))... </k>
         <modName> MId </modName>
         <typeName> TId </typeName>
         <typeGenerics> LS:List </typeGenerics>
         <typeCaps> SCS:Set </typeCaps>
    when size(LS) ==K size(TL)

    //Capability handling
    rule intersectCaps( (CS1:Set => intersectSet(CS1,CS2)) , (ListItem(type(CS2:Set,BT:BaseType)) LS:List => LS))
    rule intersectCaps( CS:Set , .List) => CS

    rule transferNonRecCaps((SetItem(R:RecCap) S1:Set => S1), S2:Set)
    rule transferNonRecCaps((SetItem(NR:NonRecCap) S1:Set => S1), (S2:Set => SetItem(NR:NonRecCap) S2) )
    rule transferNonRecCaps(.Set, S:Set) => S

    //Reuires that the set initially have the same size and are paired up correctly
    rule checkAndFilterApplies(ListItem(generic(CS1:Set, true)) LS:List, ListItem(type(CS2:Set,BT:BaseType) ) TS:List) => checkAndFilterApplies(LS, TS)
    when CS1 <=Set CS2

    rule checkAndFilterApplies(ListItem(generic(CS1:Set, false)) LS:List, ListItem(type(CS2:Set,BT:BaseType) ) TS:List) => checkAndFilterApplies(LS, TS ListItem(type(CS2,BT)))
    when CS1 <=Set CS2

    rule checkAndFilterApplies(.List, TS:List) => TS


    syntax Error ::= error(ModuleId,ErrorId)

    syntax Block ::= block(OpCodes,ValueRefs,ValueRefs)

    //Bodies / Exprs
    rule <k> Ret([CS:OpCodes],[DS:ValueRefs],[RS:ValueRefs]) => block(CS,DS,RS) ... </k>
         <frames> LS:List => ListItem(size(S)) LS </frames>
         <stack> S:List </stack>

    rule block((C:OpCode , CS:OpCodes),DS:ValueRefs,RS:ValueRefs) => C ~> block(CS,DS,RS)
    rule . ~> block(CS:OpCodes,DS:ValueRefs,RS:ValueRefs) => block(CS,DS,RS)
    rule exception(E:Error) ~> block(CS:OpCodes,DS:ValueRefs,RS:ValueRefs) => exception(E)
    rule block(.OpCodes,DS:ValueRefs,RS:ValueRefs) => processReturns(DS,RS)

    syntax K ::= push(Elem)
               | pushAll(List)
               | unlock(Set)

    rule . ~> K:K => K
    rule <k> push(E:Elem) => . ... </k>
         <stack> Stack:List => ListItem(!L:Id) Stack </stack>
         <store>  ... .Map => !L |-> E ... </store>

    rule pushAll(.List) =>.
    rule pushAll(ListElem(E:Elem) L:List) => push(E) ~> pushAll(L)

    rule unlock(.Set) => .
    rule <k> unlock(SetItem(L:Loc) S:Set) => unlock(S:Set) ... </k>
         <store> ... L|-> elem(V2:Value,T2:Type, (Lock:Int => Lock -Int 1), Bor:Set) ... </store>

    rule Let(exception(E:Error)) => exception(E)
    rule Let(return(.List)) => .
    rule Let(return(ListItem(E:Elem) RL:List)) => push(E) ~> Let(return(RL:List))

    rule <k> Copy(R:Int) => push(elem(V,type(Caps,B),0,.Set)) ... </k>
         <stack> Stack:List </stack>
         <store> ... Stack[R] |-> elem(V:Value,type(Caps:Set,B:BaseType),0, Bor:Set)  ... </store>
    when Copy in Caps

    rule <k> Fetch(R:Int) => push(elem(V,T,0,.Set)) ... </k>
         <stack> Stack:List </stack>
         <store> ... Stack[R]|-> elem(V:Value,T:Type,0,.Set) => .Map  ... </store>

    rule <k> BorrowFetch(R:Int) => push(elem(V,T,0,SetItem(Stack[R]))) ... </k>
         <stack> Stack:List </stack>
         <store> ... Stack[R] |-> elem(V:Value,T:Type, (0 => 1), Bor:Set)  ... </store>

    rule <k> Drop(R:Int) => . ... </k>
         <stack> Stack:List </stack>
         <store> ... Stack[R] |-> elem(V:Value,type(Caps:Set,B:BaseType),0,.Set) => .Map  ... </store>
    when Drop in Caps

    rule <k> Free(R:Int) => unlock(Bor) ... </k>
         <stack> Stack:List </stack>
         <store> ... (Stack[R] |-> elem(V:Value,type(Caps:Set,B:BaseType),0, Bor:Set) => .Map) ... </store>
    when notBool size(Bor) ==K 0

    rule Try(return(ES:List), [CS:Cases]) => Let(return(ES:List))

    rule Try(exception(error(MId1:Id , EId1:Id)), [(((MId2:Id . EId2:Id), E:Exp) , CS:Cases)]) => Let(E)
    when MId1 ==K MId2 andBool EId1 ==K EId2

    rule Try(exception(error(MId1:Id , EId1:Id)), [(((MId2:Id . EId2:Id), E:Exp) , CS:Cases)]) => Try(exception(error(MId1 , EId1)), [CS])
    when notBool(MId1 ==K MId2 andBool EId1 ==K EId2)

    rule Try(exception(E:Error), [.Cases]) => exception(E:Error)

       //opcodes

       //Delayed need types resolve
       //"Lit" "(" Lit "," Path ")"
       //"BorrowUnpack" "(" ValueRef "," Path ")"
       //"BorrowSwitch" "(" ValueRef "," Path "," "[" Exps "]" ")"
       // "Switch" "(" ValueRef "," Path "," "[" Exps "]" ")"
       //"BorrowPack" "(" Path "," Tag "," "[" ValueRefs "]" ")"
       //"Pack" "(" Path "," Tag "," "[" ValueRefs "]" ")"
       //"Invoke" "(" Path "," "[" ValueRefs "]" ")"

    rule <k> Unpack(R:Int, type(Caps:Set,base(MId:ModuleId, TId:TypeId, TS:Types))) => pushAll(Elems) ... </k>
         <stack> Stack:List </stack>
         <store> ... Stack[R] |-> elem(node(Tag:TagId,Elems:List),T:Type,0,.Set) => .Map ... </store>
         <curModule> MId </curModule>
         <modName> MId </modName>
         <typeName> TId </typeName>
         <ctrs> Ctrs:Bag </ctrs>
    when T ==K type(Caps,base(MId, TId, TS)) andBool Consume in Caps andBool size(Ctrs) ==K 1
    
    rule <k> Unpack(R:Int, type(Caps:Set,base(MId:ModuleId, TId:TypeId, TS:Types))) => pushAll(Elems) ... </k>
         <stack> Stack:List </stack>
         <store> ... Stack[R] |-> elem(node(Tag:TagId,Elems:List),T:Type,0,.Set) => .Map ... </store>
         <modName> MId </modName>
         <typeName> TId </typeName>
         <ctrs> Ctrs:Bag </ctrs>
    when T ==K type(Caps,base(MId, TId, TS)) andBool size(Ctrs) ==K 1
   
    rule <k> Pack(type(Caps:Set,base(MId:ModuleId, TId:TypeId, TS:Types)), Tag:TagId, [VS:Values] )) => push(Elems) ... </k>
         <modName> MId </modName>
         <typeName> TId </typeName>
         <ctrName> Tag </ctrName>
         <ctrParams> PS:List </ctrParams>
    when Create in Caps
     //todo: Owned variant
        //todo: check Types are correct
        //todo: this needs applications of them first
        //todo: Make createNodeElem function that does all that and produces the elem
        // Base need is to make type building withhin a new  <tenv> .Map </tenv>
    
    rule <k> buildCtr(PS:List,TS:List) => PS ~> restoreTEnv(TEnv) ... </k>
         <tenv> TEnv:List => TS </tenv>

    rule <k> TS:Types ~> restoreTEnv(TEnv:List) => TS ... </k>
         <tenv> _ => TEnv </tenv>

endmodule

module SANSKRIT
  imports SANSKRIT-SYNTAX
  imports SANSKRIT-SEMANTIC
  imports SANSKRIT-CONFIG

endmodule
