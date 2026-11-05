"""Typed IR for SEMPRE formulas: parse -> type-check -> execute -> verify.
LLM proposes; this module is the law. Candidates that do not parse,
type-check, or execute are rejected before affecting the world.
Subset from SEMPRE docs: (number 3), (string hi), (boolean true),
(list ...), (var x), (lambda x ...), (call + ...), map/select.
"""
from __future__ import annotations
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional

@dataclass
class Expr:
    kind: str
    value: Any = None
    args: List['Expr'] = field(default_factory=list)
    fn: str = ''
    def to_string(self) -> str:
        if self.kind == 'number':
            v = self.value
            return f'(number {int(v) if float(v) == int(v) else v})'
        if self.kind == 'string':
            s = str(self.value)
            return f'(string "{s}")' if (' ' in s or s == '') else f'(string {s})'
        if self.kind == 'boolean':
            return f"(boolean {'true' if self.value else 'false'})"
        if self.kind == 'var':
            return f'(var {self.value})'
        if self.kind == 'list':
            inner = ' '.join(a.to_string() for a in self.args)
            return f'(list {inner})' if inner else '(list)'
        if self.kind == 'lambda':
            return f'(lambda {self.value} {self.args[0].to_string()})'
        if self.kind == 'call':
            inner = ' '.join(a.to_string() for a in self.args)
            return f'(call {self.fn} {inner})' if inner else f'(call {self.fn})'
        raise ValueError(self.kind)

def tokenize(text):
    toks, i, n = [], 0, len(text)
    while i < n:
        c = text[i]
        if c.isspace():
            i += 1
        elif c in '()':
            toks.append(c); i += 1
        elif c == '"':
            j = i + 1; buf = []
            while j < n and text[j] != '"':
                if text[j] == '\\' and j + 1 < n: j += 1
                buf.append(text[j]); j += 1
            if j >= n: raise SyntaxError('unterminated string')
            toks.append('"' + ''.join(buf) + '"'); i = j + 1
        else:
            j = i
            while j < n and not text[j].isspace() and text[j] not in '()': j += 1
            toks.append(text[i:j]); i = j
    return toks

class _P:
    def __init__(self, toks): self.toks, self.i = toks, 0
    def peek(self): return self.toks[self.i] if self.i < len(self.toks) else None
    def next(self):
        t = self.peek()
        if t is None: raise SyntaxError('unexpected end of input')
        self.i += 1; return t
    def expect(self, t):
        got = self.next()
        if got != t: raise SyntaxError(f'expected {t!r} got {got!r}')

def parse(text):
    p = _P(tokenize(text)); e = _parse_expr(p)
    if p.peek() is not None: raise SyntaxError('trailing tokens')
    return e

def _parse_expr(p):
    p.expect('('); head = p.next()
    if head == 'number':
        v = p.next(); p.expect(')'); return Expr('number', float(v))
    if head == 'string':
        if p.peek() == ')': p.next(); return Expr('string', '')
        v = p.next(); p.expect(')')
        return Expr('string', v[1:-1] if v.startswith('"') else v.replace('\\ ', ' '))
    if head == 'boolean':
        v = p.next(); p.expect(')'); return Expr('boolean', v == 'true')
    if head == 'var':
        v = p.next(); p.expect(')'); return Expr('var', v)
    if head == 'list':
        items = []
        while p.peek() != ')': items.append(_parse_expr(p))
        p.next(); return Expr('list', args=items)
    if head == 'lambda':
        param = p.next(); body = _parse_expr(p); p.expect(')')
        return Expr('lambda', value=param, args=[body])
    if head == 'call':
        fn = p.next(); args = []
        while p.peek() != ')': args.append(_parse_expr(p))
        p.next(); return Expr('call', fn=fn, args=args)
    raise SyntaxError(f'unknown formula head {head!r}')

class TypeCheckError(TypeError): pass
class ExecutionError(RuntimeError): pass

def type_of(e, env=None):
    env = env or {}
    if e.kind == 'number': return 'number'
    if e.kind == 'string': return 'string'
    if e.kind == 'boolean': return 'boolean'
    if e.kind == 'var':
        if e.value not in env: raise TypeCheckError(f'unbound variable {e.value}')
        return env[e.value]
    if e.kind == 'list':
        ts = {type_of(a, env) for a in e.args}
        return f'list[{ts.pop()}]' if len(ts) == 1 else 'list[any]'
    if e.kind == 'lambda':
        bt = type_of(e.args[0], {**env, e.value: 'any'})
        return f'function[any->{bt}]'
    if e.kind == 'call':
        ts = [type_of(a, env) for a in e.args]; fn = e.fn
        if fn == '+':
            if all(t == 'number' for t in ts): return 'number'
            if all(t == 'string' for t in ts): return 'string'
            raise TypeCheckError(f'+ requires all-number or all-string, got {ts}')
        if fn in ('-', '*', '/'):
            if all(t == 'number' for t in ts): return 'number'
            raise TypeCheckError(f'{fn} requires numbers, got {ts}')
        if fn in ('<', '<=', '>', '>=', '='):
            if len(set(ts)) == 1 and ts[0] in ('number', 'string'): return 'boolean'
            raise TypeCheckError(f'{fn} requires comparable same-type args, got {ts}')
        if fn in ('and', 'or'):
            if all(t == 'boolean' for t in ts): return 'boolean'
            raise TypeCheckError(f'{fn} requires booleans, got {ts}')
        if fn == 'not':
            if ts == ['boolean']: return 'boolean'
            raise TypeCheckError(f'not requires boolean, got {ts}')
        if fn == 'if':
            if len(ts) != 3 or ts[0] != 'boolean' or ts[1] != ts[2]: raise TypeCheckError(f'if requires (boolean, T, T), got {ts}')
            return ts[1]
        if fn == 'map':
            if ts[0].startswith('list') and ts[1].startswith('function'): return 'list[' + ts[1].split('->')[1].rstrip(']') + ']'
            raise TypeCheckError(f'map requires (list, function), got {ts}')
        if fn == 'select':
            if ts[0].startswith('list') and ts[1].startswith('function'): return ts[0]
            raise TypeCheckError(f'select requires (list, function), got {ts}')
        raise TypeCheckError(f'unknown function {fn!r} (not in registry)')
    raise TypeCheckError(e.kind)

def execute(e, env=None):
    env = env or {}
    if e.kind in ('number', 'string', 'boolean'): return e.value
    if e.kind == 'var':
        if e.value not in env: raise ExecutionError(f'unbound variable {e.value}')
        return env[e.value]
    if e.kind == 'list': return [execute(a, env) for a in e.args]
    if e.kind == 'lambda':
        param, body = e.value, e.args[0]
        return lambda x: execute(body, {**env, param: x})
    if e.kind == 'call':
        fn, args = e.fn, e.args
        if fn == 'if':
            return execute(args[1], env) if execute(args[0], env) else execute(args[2], env)
        vals = [execute(a, env) for a in args]
        try:
            if fn == '+': return sum(vals) if all(isinstance(v, (int, float)) for v in vals) else ''.join(vals)
            if fn == '-': return vals[0] - sum(vals[1:])
            if fn == '*':
                r = 1
                for v in vals: r *= v
                return r
            if fn == '/': return vals[0] / vals[1]
            if fn == '<': return vals[0] < vals[1]
            if fn == '<=': return vals[0] <= vals[1]
            if fn == '>': return vals[0] > vals[1]
            if fn == '>=': return vals[0] >= vals[1]
            if fn == '=': return vals[0] == vals[1]
            if fn == 'and': return all(vals)
            if fn == 'or': return any(vals)
            if fn == 'not': return not vals[0]
            if fn == 'map': return [vals[1](x) for x in vals[0]]
            if fn == 'select': return [x for x in vals[0] if vals[1](x)]
        except Exception as ex: raise ExecutionError(str(ex)) from ex
        raise ExecutionError(f'unknown function {fn!r} (not in registry)')
    raise ExecutionError(e.kind)

@dataclass
class Verdict:
    candidate: str
    parsed: bool = False
    type_ok: bool = False
    exec_ok: bool = False
    result: Any = None
    result_type: Optional[str] = None
    error: str = ''

def verify(candidate, expected_type=None, expected_result=None, check_expected=False):
    v = Verdict(candidate=candidate)
    try: e = parse(candidate); v.parsed = True
    except Exception as ex: v.error = f'parse: {ex}'; return v
    try: v.result_type = type_of(e); v.type_ok = True
    except Exception as ex: v.error = f'type: {ex}'; return v
    if expected_type and v.result_type != expected_type:
        v.error = f'type: expected {expected_type}, got {v.result_type}'; v.type_ok = False; return v
    try: v.result = execute(e); v.exec_ok = True
    except Exception as ex: v.error = f'exec: {ex}'; return v
    if check_expected and v.result != expected_result:
        v.error = f'denotation: expected {expected_result!r}, got {v.result!r}'
    return v

def first_verified(candidates, expected_type=None):
    for c in candidates:
        v = verify(c, expected_type=expected_type)
        if v.parsed and v.type_ok and v.exec_ok and not v.error: return v
    return None
