//Todo: Note Sync is after Function Return
//      Alt we could after Blok Return

//todo: Make Capture & Restore Sync points: before and after Call
//      Ev: Make Subtask(X) Which then is handled by Dispathcer

module SANSKRIT

  imports SANSKRIT-REPR
  imports SANSKRIT-CONFIG
  imports SANSKRIT-GENERAL
  imports SANSKRIT-EXECUTE

endmodule


module SANSKRIT-REPR

  syntax Loc ::= Id         //will be generated with !L:Id //unique new var or manually put in (if manually we must ensure uniqueness)
  syntax LocList ::= List   //List of Locations

  syntax Code ::= List{OpCode,";"} //Todo: MayBe Add Return And Throw
  syntax Branch ::=

  syntax Branches ::= Map         //Map of Id -> Codes (Id may be Catch or Tag)
  syntax Bindings ::= Map         //Map of Id -> LocList (Id may be Catch or Tag)

  syntax Catches ::= Map          //Map of Ref -> Codes (Id may be Catch or Tag)

  syntax Tag ::= Id
  syntax Ref ::= Id "." Id

  syntax Type ::= "NOTImplemented"

  syntax RefMode ::= "Copy" | "Borrow" | "Move"

  syntax Capability ::= "Drop"
                    |   "Copy"
                    |   "Perisist"
                    |   "Create"
                    |   "Consume"
                    |   "Inspect"
                    |   "Embed"

  syntax Capabilities ::= Set

  syntax ExtraId ::= "RefMode"

  //Todo: Get rid of Extra and put stuff in again -- ignore where necessary
  //Todo: Seperate  Opcodes by RefMode
  syntax OpCode ::= "Extra" Map                             //its ExtraCode Augumenting the Previous one -- their is a rule that ignores Extra if not explicitly mathced
                 |  "Lit" Loc Int Int                       //todo: is value & size Extra???
                 |  "Let" LocList Code
                 //Extra: RefMode
                 |  "Fetch" Loc Loc
                 |  "Discard" Loc
                 |  "DiscardMany" LocList                   //We may remove and just expand
                 |  "DiscardBorrowed" Loc LocList
                 //Extra: RefMode, Type(Expected)
                 |  "Unpack" LocList Loc                    //Todo: More is need for type Checking -- At least the Module + Offset, Better the full Type
                 //Extra: RefMode, Type(Expected)
                 |  "Field" Loc Loc Int                     //Todo: More is need for type Checking -- At least the Module + Offset, Better the full Type
                 //Extra: RefMode, Type(Expected)
                 |  "Switch" LocList Loc Branches Bindings  //Todo: More is need for type Checking -- At least the Module + Offset, Better the full Type
                 //Extra: RefMode, Type(Expected)
                 |  "Pack" Loc LocList Tag
                 //Extra: Applies
                 |  "Invoke" LocList LocList Ref
                 |  "Try" LocList Code Catches
                 //|  "ModuleIndex"                           //Note Needs Hashing Primitive Stuff
                 |  "Image" Loc Loc
                 |  "ExtractImage" Loc Loc
                 //These are special and can only stand at the last position
                 |  "Return" LocList
                 |  "Throw" Ref
  //Basic rule that allows anybody to ignore Extra if not interested in it
  rule (Extra M:Map ; CS:Code) => CS
  //rule (. ; CS:Code) => CS
endmodule

module SANSKRIT-CONFIG
  imports SANSKRIT-REPR

  syntax Id ::= "none"

  //Shared Config
  configuration <T>
                    <tasks>
                        // 1 Task per function Call
                        <task multiplicity="*">
                            <taskId> none:Id </taskId>
                            <curModule> none:Id </curModule>
                            <curElem> none:Id </curElem>
                            <jobs>
                                //A Job is: Execution, Type Checking, capability tracking, ...
                                <job multiplicity="*">
                                    <values> .Map </values>
                                    <name> .K </name>
                                    <k> .K </k>
                                </job>
                            </jobs>
                        </task>
                    </tasks>
                    <modules>
                      <mod multiplicity="*">
                          <modName> none:Id </modName>
                          <types>
                              <type multiplicity="*">
                                  <typeName> none:Id </typeName>
                                  <typeGenerics> .List </typeGenerics>
                                  <typeCaps> .Set </typeCaps>
                                  <ctrs>
                                    <ctr multiplicity="*">
                                      <ctrName> none:Id </ctrName>
                                      <ctrParams> .List </ctrParams>
                                    </ctr>
                                  </ctrs>
                              </type>
                          </types>

                          <errors>
                              <error multiplicity="*">
                                  <errorName> none:Id </errorName>
                              </error>
                          </errors>

                          <funs>
                              <fun multiplicity="*">
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

//Has some general value tracking tools that is reusable by other modules
//this can be used in static and dnamic part
// exclusively static and exlusively dynamic tracking is in other modules
module SANSKRIT-GENERAL
  imports SANSKRIT-REPR
  imports SANSKRIT-CONFIG

  //todo: Use typed Lists
  syntax Meta ::= "none"
  syntax Value ::= Meta Values
  syntax Values ::= List
  syntax TrackerId ::= "tracker"
  syntax ProcId ::= TrackerId


  //Helper OpCodes
  syntax OpCode ::= "Push" Loc Value
                 |  "PushAll" LocList Values

  //Helpers
  syntax KResult ::= "Block" LocList Code  //Code is contiuation
                  | "Result" Values

  syntax List ::= lookupAll(List, Map)  [function]

  //General Helpers
  rule <k> lookupAll(ListItem(I:Loc) LS:List, M:Map) => ListItem(M[I]) lookupAll(LS, M) </k>

  rule <k> (Push L:Loc V:Value ; CS:Code) => CS </k>
       <values> ... .Map => L |-> Value ... </values>

  rule <k> (PushAll (ListItem(L:Loc) LS:List) (ListItem(V:Value) VS:List) ; CS:Code) =>(Push L V ; PushAll LS VS ; CS) </k>

  rule <k> (PushAll .List .List ; CS:Code) => CS </k>

  rule <k>  Result VS:Values ~> Block LS:LocList CS:Code => (PushAll LS VS; CS) </k>

  //todo: needs proper subtask Joining
  rule <k>  R:KResult ~> Restore M:Map ~> C:KResult => R ~> C </k>
       <values> _ => M </values>

  //todo: Subtask
  syntax AutoLet ::= TrackerId
  rule <k> (Let LS:LocList BC:Code ; CS:Code) => BC ~> Block LS CS </k>
       <name> _:AutoLet </name>

  syntax AutoFetch ::= TrackerId
  rule <k> (Fetch T:Loc S:Loc ; CS:Code) => (Push T V ; CS) </k>
       <values> ... S |-> V:Value ... </values>
       <name> _:AutoFetch </name>

  syntax AutoUnpack ::= TrackerId
  rule <k> (Unpack TS:LocList S:Loc ; CS:Code) => (PushAll TS VS ; CS) </k>
       <values> ... S |-> _:Meta VS:Values ... </values>
       <name> _:AutoUnpack </name>

  syntax AutoField ::= TrackerId
  rule <k> (Field T:Loc S:Loc I:Int ; CS:Code) => (Push T VS[I] ; CS) </k>
       <values> ... S |-> _:Meta VS:List ... </values>
       <name> _:AutoField </name>

  syntax AutoImage ::= TrackerId
  rule <k> (Image T:Loc S:Loc ; CS:Code) => (Fetch T S ; CS) </k>
       <name> _:AutoImage </name>

  syntax AutoExtractImage ::= TrackerId
  rule <k> (ExtractImage T:Loc S:Loc ; CS:Code) => (Fetch T S ; CS) </k>
       <name> _:AutoExtractImage </name>

  syntax AutoDiscard ::= TrackerId
  rule <k> (Discard S:Loc ; CS:Code) => CS </k>
       <name> _:AutoDiscard </name>

  syntax AutoDiscardMany ::= TrackerId
  rule <k> (DiscardMany S:LocList ; CS:Code) => CS </k>
       <name> _:AutoDiscardMany  </name>

  syntax AutoDiscardBorrowed ::= TrackerId
  rule <k> (DiscardBorrowed S:Loc B:LocList ; CS:Code) => CS </k>
       <name> _:AutoDiscardBorrowed  </name>

  //todo: Start Maintask that takes care of synchronizing Before the Call and After the Call over All Jobs
  //      It starts Subtasks and when finished replaces them with a Result

  syntax AutoInvoke ::= TrackerId
  rule <k> (Invoke PS:List RS:LocList M:Id . F:Id ; CS:Code) =>( PushAll PN lookupAll(PS,VS) ; FC) ~> Restore VS ~> Block RS CS </k>
       <values> VS:Map => .Map </values>
       <name> _:AutoInvoke </name>
       <modName> M </modName>
       <funName> F </funName>
       <funParams> PN:List </funParams>
       <funBody> FC:Code </funBody>

  syntax AutoReturn ::= TrackerId
  rule <k> (Return SS:List ; .Code) => Result lookupAll(SS,M) </k>
       <values> M:Map </values>
       <name> _:AutoReturn  </name>

endmodule




//Todo: in a second Phase have another Module that provides the base for static analysing
//todo: can we make this seperate from the real executor:
//todo:  if so the real executor can just impl the primitives
module SANSKRIT-EXECUTE
  imports SANSKRIT-REPR
  imports SANSKRIT-CONFIG
  imports SANSKRIT-GENERAL

  syntax Meta ::= "Data" Tag
                | "Primitive" Int Int

  syntax ExecuteId ::= "execute"
  syntax ProcId ::= ExecuteId

  //Helpers
  syntax KResult ::= "Block" LocList Map Code  //Like The general but with a try catch Map
                   | "Error" Ref

  //Helper
  // like the general ones but with the catch Map
  rule <k>  Result VS:Values ~> Block LS:LocList C:Map CS:Code => (PushAll LS VS; CS) </k>
       <name> execute </name>

  rule <k>  Error E:Ref ~> Block LS:LocList EM:Map CS:Code => Throw E:Ref </k>
       <name> execute </name>
       when notBool E in EM

  rule <k>  Error E:Ref ~> Block LS:LocList EM:Map CS:Code => C[E] ~> Block LS .Map CS </k>
       <name> execute </name>
       when E in EM

  //Opcodes
  //These are handled by genral code
  syntax AutoLet ::= ExecuteId
  syntax AutoFetch ::= ExecuteId
  syntax AutoUnpack ::= ExecuteId
  syntax AutoField ::= ExecuteId
  syntax AutoImage ::= ExecuteId
  syntax AutoExtractImage::= ExecuteId
  syntax AutoDiscard ::= ExecuteId
  syntax AutoDiscardMany ::= ExecuteId
  syntax AutoDiscardBorrowed ::= ExecuteId
  syntax AutoInvoke ::= ExecuteId



  //These are customly handled
  //Todo: is this necessary or does Auto work when both jobs are the same job???
    //todo: Subtask
  rule <k> (Switch LS:LocList S:Loc BC:Branches UC:Bindings; CS:Code) => (Unpack UC[I] S; BC[I]) ~> Block LS .Map CS </k>
       <values> ... S |-> Data I:Tag _ ... </values>
       <name> execute </name>

  //if we have a way to get a defualt Meta for these we can do them in Automode
  rule <k> (Lit L:Loc V:Int S:Int ; CS:Code) => CS </k>
       <values> ... .Map => L |-> Primitive V S .List ... </values>
       <name> execute </name>

  //if we have a way to get a defualt Meta for these we can do them in Automode
  rule <k> (Pack T:Loc SS:LocList I:Tag ; CS:Code) => (Push T (Data I lookupAll(SS,M)) ; CS) </k>
       <values> M:Map </values>
       <name> execute </name>

  //These are Auto implementations for Try and Switch
  //Todo: Get ??| Option 1: Have 2 Maps BC & UC, Option 2: Nest in 1 Map
  //todo: Subtask
  syntax AutoSwitch ::= ExecuteId
  rule <k> (Switch LS:LocList S:Loc BC:Branches UC:Bindings; CS:Code) => (Unpack UC[I] S; BC[I]) ~> Block LS .Map CS </k>
       <name> _:AutoSwitch </name>
       <job>
           <values> ... S |-> Data I:Tag _ ... </values>
           <name> execute </name>
       </job>

  //todo: Subtask
  syntax AutoTry ::= ExecuteId
  rule <k> (Try LS:LocList BC:Code EM:Map ; CS:Code) => BC ~> Block LS EM CS </k>
       <name> _:AutoTry </name>

  syntax AutoThrow ::= ExecuteId
  rule <k> (Throw R:Ref ; .Code) => Error R </k>
       <name> _:AutoReturn  </name>

  //Todo: How do others Throw (Especially out of Primitives)
  //Todo:   Now on a 1 + Max (overflow) the linearity checker etc. will just continue
  //Todo:    Continue is not bad as long as they go into the catch Block
  //Todo:    Maybe overwrite AutoReturn to check for Error in Block and if so give Error
  //Todo:    This would require Throw & Return to be a Sync Point
  //Todo:    Maybe even a subtask??

endmodule

