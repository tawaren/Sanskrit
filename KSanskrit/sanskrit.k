module SANSKRIT
 imports SANSKRIT-SYNTAX
 imports SANSKRIT-CONFIG
 imports SANSKRIT-PARSE
 imports SANSKRIT-REPR
endmodule


module SANSKRIT-SYNTAX
  imports ID
  imports INT

  syntax Modules ::= List{Module, ";"}
  syntax Module ::= "module" Id "{" Components "}"
  syntax Components ::= List{Component, ";"}
  syntax Component ::= Data | Signature | Function

  syntax Data ::= Accessabilities "top" Capabilities "data" Id GenericList CtrList
               |  Accessabilities Capabilities "data" Id GenericList CtrList

  syntax Signature ::= Accessabilities "transactional" Capabilities "signature" Id GenericList ParamList ":" TypedIdList
                    |  Accessabilities Capabilities "signature" Id GenericList ParamList ":" TypedIdList


  syntax Function ::= Accessabilities "transactional" "function" Id GenericList ParamList ":" TypedIdList Block
                      | Accessabilities "function" Id GenericList ParamList ":" TypedIdList Block

  syntax Implement ::= Accessabilities "implement" Id GenericList TypedIdList "for" BaseRef IdList ":" IdList Block

  syntax Accessabilities ::= List{Accessability, ""}
  syntax Accessability ::= Access PermissinList
  syntax Access ::= "global" | "local" | "guarded" GuardList

  syntax PermissinList ::= "(" Permissions ")"
  syntax Permissions ::= List{Permission, ","}
  syntax Permission ::= "create" | "consume" | "inspect" | "call" | "define"

  syntax GuardList ::= "[" Ids "]"

  syntax GenericList ::= "[" Generics "]"
  syntax Generics ::= List{Generic, ","}
  syntax Generic ::= "pahntom" Capabilities Id
                   |  Capabilities Id

  syntax Capabilities ::= List{Capability,""}
  syntax Capability ::= "Drop" | "Copy" | "Perisist" | "Primitive" | "Value" | "Unbound"

  syntax CtrList ::= "{" Ctrs "}"
  syntax Ctrs ::= List{Ctr, ";"}
  syntax Ctr ::= Id TypedIdList

  syntax TypedIdList ::= "(" TypedIds ")"
  syntax TypedIds ::= List{TypedId,","}
  syntax TypedId ::= Id ":" TypeRef

  syntax BaseRef ::= Id "." Id TypeRefArgs
                   | "." Id TypeRefArgs
  syntax TypeRefArgs ::= "[" TypeRefs "]"
  syntax TypeRefs ::= List{TypeRef, ","}
  syntax TypeRef ::= Id | BaseRef | "project" TypeRef

  syntax ParamList ::= "(" Params ")"
  syntax Params ::= List{Param,","}
  syntax Param ::= "consume" Id ":" TypeRef
                |  Id ":" TypeRef

  syntax Args ::= List{Arg,","}
  syntax Arg ::= Loc

  syntax Block ::= "{" Code "}"
  syntax Code ::= List{OpCode, ";"}
  syntax OpCode ::= PlainOpCode | Ids = PlainOpCode
  syntax IdList ::= "(" Ids ")"
  syntax Ids ::= List{Id, ","}

  syntax Lit ::= Int
  syntax Loc ::= Int | Id
  syntax Pos ::= Int | Id

  syntax Branches ::= List{Branch,""}
  syntax Branch ::= "case" Id Block
                  | "case" Id IdList Block

  syntax TryArgs::= List{TryArg,""}
  syntax TryArg ::= "!" Loc | Loc

  syntax PlainOpCode ::= "Lit" BaseRef Lit
                      | "Let" Block
                      | "Copy" Args
                      | "Move" Args
                      | "Discard" Args
                      | "Copy" "Unpack" Loc
                      | "Move" "Unpack" Loc
                      | "Copy" "Field" Loc Pos
                      | "Move" "Field" Loc Pos
                      | "Copy" "Switch" Loc Branches
                      | "Move" "Switch" Loc Branches
                      | "Inspect" Loc Branches
                      | "Invoke" BaseRef Args
                      | "Sig" "Invoke" Loc Args
                      | "Try" "Invoke" BaseRef TryArgs Branch Branch
                      | "Try" "Sig" "Invoke" Loc TryArgs Branch Branch
                      | "Project" Loc
                      | "Unproject" Loc
                      | "Rollback" Args ":" TypeRefArgs
endmodule

module SANSKRIT-CONFIG
  imports ID
  imports MAP
  imports LIST
  imports SET
  imports SANSKRIT-SYNTAX

  syntax Id ::= "none"
  syntax ElemType ::= "file" | "components" | "data" | "signature" | "function" | "implement"

  //Shared Config
  configuration <T>
                    <tasks>
                        <task multiplicity="*" type="Set">
                            <taskId> none:Id </taskId>
                            <curModule> none:Id </curModule>
                            <curElem> none:Id </curElem>
                            <elemType> file:ElemType </elemType>
                            <k> $PGM:Modules </k>
                        </task>
                    </tasks>
                    <modules>
                      <mod multiplicity="*" type="Set">
                          <modName> none:Id </modName>
                          <types>
                              <type multiplicity="*" type="Set">
                                  <typeName> none:Id </typeName>
                                  <typeGenerics> .List </typeGenerics>
                                  <typeCaps> .Set </typeCaps>
                                  <ctrs>
                                    <ctr multiplicity="*" type="Set">
                                      <ctrName> none:Id </ctrName>
                                      <ctrParams> .List </ctrParams>
                                    </ctr>
                                  </ctrs>
                              </type>
                          </types>

                          <funs>
                              <fun multiplicity="*" type="Set">
                                  <funName> none:Id </funName>
                                  <funGenerics> .List </funGenerics>
                                  // <funVisibility> private </funVisibility>
                                  <funRisks> .Set </funRisks>
                                  <funParams> .List </funParams>
                                  <funReturns> .List </funReturns>
                                  <funBody> .Code </funBody>
                              </fun>
                          </funs>
                      </mod>
                    </modules>
                </T>
endmodule


module SANSKRIT-REPR
    imports ID
    imports SET
    imports MAP
    imports LIST
    imports BOOL
    imports SANSKRIT-CONFIG

    syntax ComponentDecl ::= data(Map,Bool,Set,List,Map)
                           | signature(Map,Bool,Set,List,List,List)
                           | function(Map,Bool,List,List,List,List)
                           | implement(Map,List,List, ??,List,List,List)

    syntax Type ::= genType(Id)                             //Generic
                  | compType(ModRef,Id,List)    [strict(1)] //From Module
                  | projType(Type)                          //Projected


    syntax CompRef ::= compRef(ModRef,Id,List) [strict(1)]


    syntax Field ::= field(Id,Type)
    syntax ParamDef ::= param(Bool,Id,Type)
    syntax Return ::= return(Id,Type)

    syntax OpCaodeDef ::= lit(Assig, CompRef, Lit) //todo: only 1 assig?
                        | let(Assig, List)
                        | copy(Assig, List)  //todo: only 1 assig?
                        | move(Assig, List)  //todo: only 1 assig?
                        | discard(List)
                        | copy_unpack(Assig, Loc) //todo: alt for loc/pos?
                        | move_unpack(Assig, Loc) //todo: alt for loc/pos?
                        | copy_field(Assig, Loc, Pos) //todo: alt for loc/pos?
                        | move_field(Assig, Loc, Pos) //todo: alt for loc/pos?
                        | copy_switch(Assig, Loc, Branches) //Todo: alt structure for branches
                        | move_switch(Assig, Loc, Branches) //Todo: alt structure for branches
                        | inspect(Assig, Loc, Branches) //Todo: alt structure for branches
                        | invoke(Assig, CompRef, List)
                        | sig_invoke(Assig, Loc, List)  //todo: alt for loc/pos?
                        | try_invoke(Assig, CompRef, List, Branch, Branch) //todo: alt for Branches
                        | try_sig_invoke(Assig, Loc, List, Branch, Branch)  //todo: alt for Branches
                        | project(Assig, List)  //todo: only 1 assig?
                        | unproject(Assig, List)  //todo: only 1 assig?
                        | rollback(Assig, List, List)

    syntax Assig ::= List |  "none"

    //eager resolution of this
    syntax ModRef ::= Id | "this"
    syntax KResult ::= Id

    rule <k> this => M ... </k>
         <curModule> M </curModule>

endmodule

module SANSKRIT-PARSE
    imports SANSKRIT-CONFIG
    imports SANSKRIT-SYNTAX
    imports SANSKRIT-REPR
    imports SET
    imports MAP
    imports LIST
    imports BOOL

  rule <k> module N:Id { CS:Components } ; ML:Modules => ML </k>
       <elemType> file </elemType>
       (.Bag => <task> ...
            <curModule> N </curModule>
            <elemType> components:ElemType </elemType>
            <k> CS </k>
        ... </task>)
       (.Bag => <mod> ... <modName> N </modName> ... </mod>)

  //Data
  rule <k> AS:Accessabilities top CAPS:Capabilities data N:Id GS:GenericList CTRS:CtrList ; CS:Components => CS </k>
       <curModule> M </curModule>
       <elemType> components </elemType>
       (.Bag => <task> ...
           <curModule> M </curModule>
           <curElem> N </curElem>
           <elemType> data:ElemType </elemType>
           <k> data(toAccessMap(AS, .Map), true, toCapSet(CAPS, .Set), toGenericList(GS, .List), toCtrMap(CTRS, .Map)) </k>
       ... </task>)

  rule <k> AS:Accessabilities CAPS:Capabilities data N:Id GS:GenericList CTRS:CtrList ; CS:Components => CS </k>
       <curModule> M </curModule>
       <elemType> components </elemType>
       (.Bag => <task> ...
           <curModule> M </curModule>
           <curElem> N </curElem>
           <elemType> data:ElemType </elemType>
           <k> data(toAccessMap(AS, .Map), false, toCapSet(CAPS, .Set), toGenericList(GS, .List), toCtrMap(CTRS, .Map)) </k>
       ... </task>)

  //Signature
  rule <k> AS:Accessabilities transactional CAPS:Capabilities signature N:Id GS:GenericList PL:ParamList : RL:TypedIdList ; CS:Components => CS </k>
         <curModule> M </curModule>
         <elemType> components </elemType>
         (.Bag => <task> ...
             <curModule> M </curModule>
             <curElem> N </curElem>
             <elemType> signature:ElemType </elemType>
             <k> signature(toAccessMap(AS, .Map),true,toCapSet(CAPS, .Set),toGenericList(GS, .List),toParamList(PL, .List),toReturnList(RL, .List))  </k>
         ... </task>)

  rule <k> AS:Accessabilities CAPS:Capabilities signature N:Id GS:GenericList PL:ParamList : RL:TypedIdList ; CS:Components => CS </k>
         <curModule> M </curModule>
         <elemType> components </elemType>
         (.Bag => <task> ...
             <curModule> M </curModule>
             <curElem> N </curElem>
             <elemType> signature:ElemType </elemType>
             <k> signature(toAccessMap(AS, .Map),false,toCapSet(CAPS, .Set),toGenericList(GS, .List),toParamList(PL, .List),toReturnList(RL, .List))  </k>
         ... </task>)

  //Function
   rule <k> AS:Accessabilities transactional function N:Id GS:GenericList PL:ParamList : RL:TypedIdList B:Block ; CS:Components => CS </k>
           <curModule> M </curModule>
           <elemType> components </elemType>
           (.Bag => <task> ...
               <curModule> M </curModule>
               <curElem> N </curElem>
               <elemType> function:ElemType </elemType>
               <k> function(toAccessMap(AS, .Map),true,toGenericList(GS, .List),toParamList(PL, .List),toReturnList(RL, .List),  toOpCodeList(B, .List))  </k>
           ... </task>)

   rule <k> AS:Accessabilities function N:Id GS:GenericList PL:ParamList : RL:TypedIdList B:Block ; CS:Components => CS </k>
           <curModule> M </curModule>
           <elemType> components </elemType>
           (.Bag => <task> ...
               <curModule> M </curModule>
               <curElem> N </curElem>
               <elemType> function:ElemType </elemType>
               <k> function(toAccessMap(AS, .Map),false,toGenericList(GS, .List),toParamList(PL, .List),toReturnList(RL, .List), toOpCodeList(B, .List))  </k>
           ... </task>)

  // Implement
  //  syntax Implement ::= Accessabilities "implement" Id GenericList TypedIdList "for" BaseRef IdList ":" IdList Block


  syntax Map ::= toAccessMap(Accessabilities, Map) [function]
               | toCtrMap(CtrList, Map) [function]

  syntax List ::= toGenericList(GenericList, List) [function]
                | toFieldList(TypedIdList, List) [function]
                | toReturnList(TypedIdList, List) [function]
                | toTypeList(TypeRefArgs, List) [function]
                | toParamList(ParamList, List) [function]
                | toOpCodeList(Block, List) [function]
                | toIdList(Ids) [function]
                | toArgList(Args) [function]

  syntax Set ::= toCapSet(Capabilities, Set) [function]

  syntax Field ::= toField(TypedId) [function]
  syntax Type ::= toType(TypeRef) [function]
  syntax CompRef ::= toCompRef(BaseRef) [function]
  syntax ParamDef ::= toParam(Param) [function]
  syntax Return ::= toReturn(TypedId) [function]
  syntax OpCodeDef ::= toOpCode(OpCode) [function]


  //Todo: prevent when already in
  rule toGenericList([G:Generic, GS:Generics], LS:List) => toGenericList([GS], LS ListItem(G))
  rule toGenericList([.Generics], LS:List) => LS

  //Todo: prevent when already in
  rule toCtrMap({K:Id V:TypedIdList; CS:Ctrs}, CM:Map) => toCtrMap({CS}, CM (K |-> toFieldList(V,.List)))
  rule toCtrMap({.Ctrs}, CM:Map) => CM

  rule toFieldList((V:TypedId , FS:TypedIds), LS:List) => toFieldList((FS), LS ListItem(toField(V)))
  rule toFieldList((.TypedIds), LS:List) => LS

  rule toField(I:Id : TR:TypeRef) => field(I, toType(TR))

  rule toParamList((P:Param , PS:Params), LS:List) => toParamList((PS), LS ListItem(toParam(P)))
  rule toParamList((.Params), LS:List) => LS

  rule toParam(consume I:Id : TR:TypeRef) => param(true,I, toType(TR))
  rule toParam(I:Id : TR:TypeRef) => param(false,I, toType(TR))

  rule toReturnList((R:TypedId , RS:TypedIds), LS:List) => toReturnList((RS), LS ListItem(toReturn(R)))
  rule toReturnList((.TypedIds), LS:List) => LS

  rule toReturn(I:Id : TR:TypeRef) => return(I, toType(TR))

  rule toOpCodeList({OC:OpCode ; C:Code}, LS:List) => toOpCodeList({C}, LS ListItem(toOpCode(OC)))
  rule toOpCodeList({.Code}, LS:List) => LS

  rule toType(I:Id) => genType(I)
  rule toType(project TR:TypeRef) => projType(toType(TR))
  rule toType(M:Id . C:Id TRA:TypeRefArgs) => compType(M, C, toTypeList(TRA, .List))
  rule toType(. C:Id TRA:TypeRefArgs) => compType(this, C, toTypeList(TRA, .List))

  rule toTypeList([T:TypeRef, TS:TypeRefs], L:List) =>  toTypeList([TS:TypeRefs], L ListItem(toType(T)))
  rule toTypeList([.TypeRefs], L:List) => L

  rule toCompRef(M:Id . C:Id TRA:TypeRefArgs) => compRef(M, C, toTypeList(TRA, .List))

  rule toIdList([I:Id, IS:Ids], L:List) =>  toIdList([IS], L ListItem(I))
  rule toIdList([.Ids], L:List) => L

  rule toArgList([LO:Loc, AS:Args], L:List) =>  toArgList([AS], L ListItem(LO))
  rule toArgList([.Args], L:List) => L

  //  syntax OpCode ::= PlainOpCode | PlainOpCode "=>" Ids

  //Todo: prevent when already in
  rule toCapSet(C:Capability CS:Capabilities, SS:Set) => toCapSet(CS, SS SetItem(C))
  rule toCapSet(.Capabilities, SS:Set) => SS

  //Todo: prevent when already in
  rule toAccessMap(A:Access ( P:Permission, PS:Permissions ) AS:Accessabilities,M) => toAccessMap(global ( PS ) AS, M (P |-> A))
  rule toAccessMap(_:Access ( .Permissions ) AS:Accessabilities,M) => toAccessMap(AS:Accessabilities, M )
  rule toAccessMap(.Accessabilities,M) => M

  rule Lit BR:BaseRef L:Lit => lit(none, toCompRef(BR), L)
  rule IS:Ids = Lit BR:BaseRef L:Lit => lit(toIdList(IS,.List), toCompRef(BR), L)

  rule Let B:Block => let(none, toOpCodeList(B, .List))
  rule IS:Ids = Let B:Block => let(toIdList(IS,.List), toOpCodeList(B, .List))

  rule Copy AS:Args => copy(none, toArgList(AS, .List))
  rule IS:Ids = Copy AS:Args => copy(toIdList(IS,.List), toArgList(AS, .List))

  rule Move AS:Args => move(none, toArgList(AS, .List))
  rule IS:Ids = Move AS:Args => move(toIdList(IS,.List), toArgList(AS, .List))

  rule Discard AS:Args => discard(toArgList(AS, .List))

  rule Copy Unpack L:Loc => copy_unpack(none, L)
  rule IS:Ids = Copy Unpack L:Loc => copy_unpack(toIdList(IS,.List), L)

  rule Move Unpack L:Loc => move_unpack(none, L)
  rule IS:Ids = Move Unpack L:Loc => move_unpack(toIdList(IS,.List), L)

  rule Copy Field L:Loc P:Pos => copy_field(none, L, P)
  rule IS:Ids = Copy Field L:Loc P:Pos => copy_field(toIdList(IS,.List), L, P)

  rule Move Field L:Loc P:Pos => move_field(none, L, P)
  rule IS:Ids = Move Field L:Loc P:Pos => move_field(toIdList(IS,.List), L, P)


  syntax OpCode ::= PlainOpCode | Ids "=" PlainOpCode

  syntax PlainOpCode ::= "Copy" "Switch" Loc Branches
                       | "Move" "Switch" Loc Branches
                       | "Inspect" Loc Branches
                       | "Invoke" BaseRef Args
                       | "Sig" "Invoke" Loc Args
                       | "Try" "Invoke" BaseRef TryArgs Branch Branch
                       | "Try" "Sig" "Invoke" Loc TryArgs Branch Branch
                       | "Project" Loc
                       | "Unproject" Loc
                       | "Rollback" Args ":" TypeRefArgs


      syntax OpCaodeDef ::= copy_switch(Assig, Loc, Branches) //Todo: alt structure for branches
                          | move_switch(Assig, Loc, Branches) //Todo: alt structure for branches
                          | inspect(Assig, Loc, Branches) //Todo: alt structure for branches
                          | invoke(Assig, CompRef, List)
                          | sig_invoke(Assig, Loc, List)  //todo: alt for loc/pos?
                          | try_invoke(Assig, CompRef, List, Branch, Branch) //todo: alt for Branches
                          | try_sig_invoke(Assig, Loc, List, Branch, Branch)  //todo: alt for Branches
                          | project(Assig, List)  //todo: only 1 assig?
                          | unproject(Assig, List)  //todo: only 1 assig?
                          | rollback(Assig, List, List)

endmodule
