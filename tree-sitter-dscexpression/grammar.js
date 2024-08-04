const PREC = {
  ESCAPEDSTRING: 3,
  BRACKETINSTRING: 2,
  EXPRESSIONSTRING: 1,
  STRINGLITERAL: -11,
}

module.exports = grammar({
  name: 'dscexpression',

  rules: {
    statement: $ => choice(
      $.escapedStringLiteral,
      $.bracketInStringLiteral,
      $._expressionString,
      $.stringLiteral,
    ),
    escapedStringLiteral: $ => token(prec(PREC.ESCAPEDSTRING, seq('[[', /.*?/))),
    bracketInStringLiteral: $ => token(prec(PREC.BRACKETINSTRING, seq('[', /.*?/, ']', /.+?/))),
    _expressionString: $ => prec(PREC.EXPRESSIONSTRING, seq('[', $.expression, ']')),
    expression: $ => seq(field('function', $.function), optional($.accessor)),
    stringLiteral: $ => token(prec(PREC.STRINGLITERAL, /[^\[].*?/)),

    function: $ => seq(field('name', $.functionName), '(', field('args', optional($.arguments)), ')'),
    functionName: $ => /[a-z][a-zA-Z0-9]*/,
    arguments: $ => seq($._argument, repeat(seq(',', $._argument))),
    _argument: $ => choice($.expression, $._quotedString, $.number, $.boolean),

    _quotedString: $ => seq('\'', $.string, '\''),
    // ARM strings are allowed to contain single-quote characters
    string: $ => /[^']*/,
    number: $ => /-?\d+/,
    boolean: $ => choice('true', 'false'),

    accessor: $ => choice(field('members', $.memberAccess), field('index', $.arrayIndex)),

    memberAccess: $ => seq('.', $.memberName, optional($.accessor)),
    memberName: $ => /[a-zA-Z0-9_-]+/,

    arrayIndex: $ => seq('[', $.expression, ']'),
  }

});
