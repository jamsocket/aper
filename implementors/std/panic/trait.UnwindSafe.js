(function() {var implementors = {};
implementors["aper"] = [{"text":"impl&lt;Transition&gt; UnwindSafe for SuspendedEvent&lt;Transition&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Transition: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;Transition&gt; UnwindSafe for TransitionEvent&lt;Transition&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Transition: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl UnwindSafe for PlayerID","synthetic":true,"types":[]},{"text":"impl&lt;State&gt; UnwindSafe for StateUpdateMessage&lt;State&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;State: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;State as StateMachine&gt;::Transition: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]}];
implementors["aper_actix"] = [{"text":"impl&lt;State&gt; !UnwindSafe for ChannelActor&lt;State&gt;","synthetic":true,"types":[]},{"text":"impl&lt;State&gt; UnwindSafe for WrappedStateUpdateMessage&lt;State&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;State: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;State as StateMachine&gt;::Transition: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;State&gt; !UnwindSafe for PlayerActor&lt;State&gt;","synthetic":true,"types":[]},{"text":"impl UnwindSafe for CreateChannelMessage","synthetic":true,"types":[]},{"text":"impl&lt;State&gt; UnwindSafe for GetChannelMessage&lt;State&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;State: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;State&gt; !UnwindSafe for ServerActor&lt;State&gt;","synthetic":true,"types":[]},{"text":"impl&lt;State&gt; !UnwindSafe for ChannelMessage&lt;State&gt;","synthetic":true,"types":[]}];
implementors["aper_yew"] = [{"text":"impl&lt;State&gt; UnwindSafe for StateManager&lt;State&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;State: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;View&gt; !UnwindSafe for Props&lt;View&gt;","synthetic":true,"types":[]},{"text":"impl&lt;View&gt; !UnwindSafe for StateMachineComponent&lt;View&gt;","synthetic":true,"types":[]},{"text":"impl&lt;State&gt; !UnwindSafe for ViewContext&lt;State&gt;","synthetic":true,"types":[]},{"text":"impl&lt;State&gt; UnwindSafe for Status&lt;State&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;State: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl&lt;State&gt; UnwindSafe for Msg&lt;State&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;State: UnwindSafe,<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;State as StateMachine&gt;::Transition: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()