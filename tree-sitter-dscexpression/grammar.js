module.exports = grammar({
  name: 'dscexpression',

  rules: {
    statement: $ => choice(
      $.escapedStringLiteral,
      $.expression,
      $.stringLiteral,
    ),
    escapedStringLiteral: $ => token(seq('[[', /.*/)),
    expression: $ => seq('[', $.function, optional($._members), ']'),
    stringLiteral: $ => token(prec(-11, /.*/)),
    function: $ => seq($.functionName, '(', optional($._arguments), ')'),
    functionName: $ => /[a-zA-Z]+/,
    _arguments: $ => seq($._argument, repeat(seq(',', $._argument))),
    _argument: $ => choice($.function, $.string, $.number, $.boolean),
    string: $ => seq("'", /[^']*/, "'"),
    number: $ => /\d+/,
    boolean: $ => choice('true', 'false'),
    _members: $ => repeat1($._member),
    _member: $ => seq('.', $.memberName),
    memberName: $ => /[a-zA-Z0-9_-]+/,
  }

});
